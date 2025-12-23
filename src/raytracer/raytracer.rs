use crate::imgcomparator::Image;
use crate::raytracer::config::light::Light::{Directional, Point};
use crate::raytracer::config::Config;
use crate::raytracer::config::Ray;
use rayon::prelude::*;
use bvh::bvh::Bvh;
use bvh::bounding_hierarchy::BoundingHierarchy;
use nalgebra::{Point3, Vector3};

/// RayTracer with BVH (Bounding Volume Hierarchy) acceleration structure.
/// 
/// The BVH organizes scene objects into a binary tree based on their spatial positions,
/// enabling efficient ray-object intersection tests. Instead of testing against all objects
/// (O(n) complexity), the BVH reduces this to O(log n) on average by quickly culling
/// large portions of the scene that a ray cannot intersect.
pub struct RayTracer {
    config: Config,
    /// BVH acceleration structure for fast ray-object intersection queries.
    /// Built once during initialization using Surface Area Heuristic (SAH) for optimal partitioning.
    bvh: Bvh<f32, 3>,
}

impl RayTracer {
    /// Creates a new RayTracer and builds the BVH acceleration structure.
    /// 
    /// The BVH is constructed using parallel processing (via rayon) for better performance
    /// with large scenes. The construction uses SAH (Surface Area Heuristic) to determine
    /// optimal split planes, resulting in efficient traversal during rendering.
    pub fn new(mut config: Config) -> Self {
        // Build BVH from scene objects using parallel construction
        let mut objects = config.get_scene_objects().clone();
        let bvh = Bvh::build_par(&mut objects);
        
        // Update the config with the modified objects (they now have BVH indices)
        *config.get_scene_objects_mut() = objects;
        
        RayTracer { config, bvh }
    }

pub fn render(&self) -> Result<Image, String> {
    let width = self.config.width as usize;
    let height = self.config.height as usize;
    
    let mut image_data = vec![0u32; width * height];

    let camera_vector = self.config.camera.direction().normalize();
    let normal_to_plane = camera_vector.cross(&self.config.camera.up).normalize();
    let v = normal_to_plane.cross(&camera_vector).normalize();
    
    let fovrad = self.config.camera.fov * std::f32::consts::PI / 180.0;
    let pixel_height = (fovrad / 2.0).tan();
    let pixel_width = pixel_height * (self.config.width as f32 / self.config.height as f32);

    let img_width_by_2 = self.config.width as f32 / 2.0;
    let img_height_by_2 = self.config.height as f32 / 2.0;

    image_data.par_chunks_mut(width)
        .enumerate()
        .for_each(|(y, row)| {
            
            let b = (pixel_height * (img_height_by_2 - (y as f32 + 0.5))) / img_height_by_2;

            for (x, pixel) in row.iter_mut().enumerate() {
                let a = (pixel_width * ((x as f32 + 0.5) - img_width_by_2)) / img_width_by_2;
                
                let d = (normal_to_plane * a + v * b + camera_vector).normalize();
                
                let color = self.find_color(self.config.camera.position, d);
                
                *pixel = color;
            }
        });

    Ok(Image::new(
        self.config.width,
        self.config.height,
        image_data,
    ))
}

    pub fn get_output_path(&self) -> &str {
        &self.config.output_file
    }

    fn find_color(&self, origin: Vector3<f32>, direction: Vector3<f32>) -> u32 {
        let color_vec = self.find_color_recursive(origin, direction, 0);
        let r = (color_vec.x.max(0.0).min(1.0) * 255.0).round() as u32;
        let g = (color_vec.y.max(0.0).min(1.0) * 255.0).round() as u32;
        let b = (color_vec.z.max(0.0).min(1.0) * 255.0).round() as u32;
        (r << 16) | (g << 8) | b
    }

    /// Helper function to create a BVH ray from Vector3 origin and direction.
    fn create_bvh_ray(origin: Vector3<f32>, direction: Vector3<f32>) -> bvh::ray::Ray<f32, 3> {
        let origin_point = Point3::from(origin);
        bvh::ray::Ray::new(origin_point, direction)
    }

    fn find_color_recursive(&self, origin: Vector3<f32>, direction: Vector3<f32>, depth: u32) -> Vector3<f32> {
        if depth > self.config.maxdepth {
            return Vector3::zeros();
        }
        
        let ray: Ray = Ray { origin, direction };
        
        // Use BVH to get candidate objects that the ray might intersect.
        // This is the key optimization: instead of testing all objects, the BVH
        // quickly identifies only the objects whose bounding boxes intersect the ray.
        let bvh_ray = Self::create_bvh_ray(origin, direction);
        let candidates = self.bvh.traverse(&bvh_ray, self.config.get_scene_objects());
        
        // Find closest intersection among candidates returned by BVH
        let closest_intersection = candidates
            .iter()
            .filter_map(|object| object.intersect(&ray))
            .min_by(|a, b| {
                a.distance
                    .partial_cmp(&b.distance)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            
        if let Some(intersection) = closest_intersection {
            // Accumulate light contributions from all light sources
            let mut light_accumulator = Vector3::zeros();
            
            for light in self.config.get_lights() {
                // shadow ray
                let light_dir = match light {
                    Point { position, .. } => (*position - intersection.point).normalize(),
                    Directional { direction, .. } => *direction,
                };
                let shadow_ray = Ray {
                    origin: intersection.point + intersection.normal * 1e-6,
                    direction: light_dir,
                };
                
                // Use BVH for shadow ray testing. This is particularly beneficial for complex
                // scenes with many objects, as shadow rays are cast for every intersection point
                // and every light source. BVH drastically reduces the number of intersection tests.
                let shadow_bvh_ray = Self::create_bvh_ray(shadow_ray.origin, shadow_ray.direction);
                let shadow_candidates = self.bvh.traverse(&shadow_bvh_ray, self.config.get_scene_objects());
                
                let in_shadow = shadow_candidates
                    .iter()
                    .filter_map(|object| object.intersect(&shadow_ray))
                    .any(|shadow_intersection| {
                        if shadow_intersection.distance < 1e-6 {
                            return false;
                        }
                        if intersection.is_back_face && shadow_intersection.is_back_face {
                            return false;
                        }
                        match light {
                            Point { position, .. } => {
                                shadow_intersection.distance < (*position - intersection.point).norm()
                            }
                            Directional { .. } => true,
                        }
                    });
                if !in_shadow {
                    let light_color = light.color();
                    let n_dot_l = intersection.normal.dot(&light_dir).max(0.0);
                    let diffuse = intersection.diffuse_color * n_dot_l;
                    let view_dir = -direction;
                    let half_vector = (light_dir + view_dir).normalize();
                    let n_dot_h = intersection.normal.dot(&half_vector).max(0.0);
                    
                    let specular_factor = if intersection.shininess == 1.0 {
                        n_dot_h
                    } else if intersection.shininess == 0.0 {
                        if n_dot_l > 0.0 { n_dot_h } else { 0.0 }
                    } else {
                        if n_dot_l > 0.0 { n_dot_h.powf(intersection.shininess) } else { 0.0 }
                    };
                    
                    let specular = intersection.specular_color * specular_factor;
                    light_accumulator += (diffuse + specular).component_mul(&light_color);
                }
            }
            
            let mut final_color = light_accumulator + self.config.ambient;
            
            let is_reflective = intersection.specular_color.x > 0.0 
                || intersection.specular_color.y > 0.0 
                || intersection.specular_color.z > 0.0;
            
            if is_reflective && depth + 1 < self.config.maxdepth {
                let reflect_dir = direction - 2.0 * direction.dot(&intersection.normal) * intersection.normal;
                
                let reflect_origin = intersection.point + intersection.normal * 1e-6;
                
                let reflected_color = self.find_color_recursive(reflect_origin, reflect_dir, depth + 1);
                
                let reflection_contribution = intersection.specular_color.component_mul(&reflected_color);
                final_color += reflection_contribution;
            }
            
            final_color
        } else {
            Vector3::zeros()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::imgcomparator::file_to_image;
    use crate::imgcomparator::save_image;
    use crate::imgcomparator::Image;
    use crate::raytracer::ParsedConfigState;

    const SAVE_DIFF_IMAGES: bool = true;

    #[test]
    fn test_raytracer_tp31() {
        test_file("test_file/jalon3/tp31");
    }
    #[test]
    fn test_raytracer_tp32() {
        test_file("test_file/jalon3/tp32");
    }

    #[test]
    fn test_raytracer_tp33() {
        test_file("test_file/jalon3/tp33");
    }

    #[test]
    fn test_raytracer_tp34() {
        test_file("test_file/jalon3/tp34");
    }

    #[test]
    fn test_raytracer_tp35() {
        test_file("test_file/jalon3/tp35");
    }

    #[test]
    fn test_raytracer_tp51diffuse() {
        test_file("test_file/jalon5/tp51-diffuse");
    }

    #[test]
    fn test_raytracer_tp51specular() {
        test_file("test_file/jalon5/tp51-specular");
    }

    #[test]
    fn test_raytracer_tp52() {
        test_file("test_file/jalon5/tp52");
    }

    #[test]
    fn test_raytracer_tp53() {
        test_file("test_file/jalon5/tp53");
    }

    #[test]
    fn test_raytracer_tp54() {
        test_file("test_file/jalon5/tp54");
    }

    #[test]
    fn test_raytracer_tp55() {
        test_file("test_file/jalon5/tp55");
    }

    #[test]
    fn test_raytracer_tp61directional() {
        test_file("test_file/jalon6/tp61-dir");
    }

    #[test]
    fn test_raytracer_tp61() {
        test_file("test_file/jalon6/tp61");
    }

    #[test]
    fn test_raytracer_tp62_1() {
        test_file("test_file/jalon6/tp62-1");
    }

    #[test]
    fn test_raytracer_tp62_2() {
        test_file("test_file/jalon6/tp62-2");
    }

    #[test]
    fn test_raytracer_tp62_3() {
        test_file("test_file/jalon6/tp62-3");
    }

    #[test]
    fn test_raytracer_tp62_4() {
        test_file("test_file/jalon6/tp62-4");
    }

    #[test]
    fn test_raytracer_tp62_5() {
        test_file("test_file/jalon6/tp62-5");
    }

    #[test]
    fn test_raytracer_tp63() {
        test_file("test_file/jalon6/tp63");
    }

    #[test]
    fn test_raytracer_tp64() {
        test_file("test_file/jalon6/tp64");
    }

    /// Benchmark test to demonstrate BVH performance improvement.
    /// This test measures rendering time and logs it for comparison.
    #[test]
    fn test_bvh_performance_benchmark() {
        // Use a complex scene for benchmarking
        let scene_file = "test_file/jalon6/tp64.test";
        let mut parsed_config = ParsedConfigState::new();
        let config = parsed_config
            .load_config_file(&scene_file)
            .expect("Failed to load configuration");
        
        let object_count = config.get_scene_objects().len();
        println!("\n=== BVH Performance Benchmark ===");
        println!("Scene: {}", scene_file);
        println!("Number of objects: {}", object_count);
        
        // Benchmark with BVH
        let ray_tracer = RayTracer::new(config);
        let start_time = std::time::Instant::now();
        let _result = ray_tracer.render().expect("Failed to render image");
        let duration = start_time.elapsed();
        
        println!("Render time with BVH: {:?}", duration);
        println!("Expected speedup: O(log n) vs O(n) for {} objects", object_count);
        println!("Theoretical complexity: O(log₂({})) ≈ {:.1} vs O({})", 
                 object_count, 
                 (object_count as f64).log2(), 
                 object_count);
        println!("=================================\n");
        
        // The test passes if rendering completes successfully
        assert!(duration.as_secs() < 300, "Rendering took too long (>5 minutes)");
    }


    fn test_file(path: &str) {
        let scene_file = format!("{path}.test");
        let expected_image_file = format!("{path}.png");
        let mut parsed_config = ParsedConfigState::new();
        let config = parsed_config
            .load_config_file(&scene_file)
            .expect("Failed to load configuration");
        let ray_tracer = RayTracer::new(config);
        let generated_image = ray_tracer.render().expect("Failed to render image");
        let expected_image =
            file_to_image(&expected_image_file).expect("Failed to load expected image");
        let (diff, img) =
            Image::compare(&generated_image, &expected_image).expect("Failed to compare images");
        if SAVE_DIFF_IMAGES {
            let diff_image_path = format!("{path}_diff.png");
            save_image(&img, &diff_image_path).expect("Failed to save diff image");
            let generated_image_path = format!("{path}_generated.png");
            save_image(&generated_image, &generated_image_path)
                .expect("Failed to save generated image");
        }
        assert_eq!(diff, 0, "Images differ! See {path}_diff.png for details.");
    }
}
