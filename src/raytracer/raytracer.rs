use crate::imgcomparator::Image;
use crate::raytracer::config::light::Light::{Directional, Point};
use crate::raytracer::config::Config;
use crate::raytracer::config::Ray;
use rayon::prelude::*; // Ensure this is imported

pub struct RayTracer {
    config: Config,
}

impl RayTracer {
    pub fn new(config: Config) -> Self {
        RayTracer { config }
    }

pub fn render(&self) -> Result<Image, String> {
    let width = self.config.width as usize;
    let height = self.config.height as usize;
    
    // Create the buffer
    let mut image_data = vec![0u32; width * height];

    // --- Pre-calculation (Unchanged) ---
    // We calculate these once on the main thread to avoid re-doing math in every thread.
    let camera_vector = self.config.camera.direction().normalize();
    let normal_to_plane = camera_vector.cross(self.config.camera.up).normalize();
    let v = normal_to_plane.cross(camera_vector).normalize();
    
    let fovrad = self.config.camera.fov * std::f32::consts::PI / 180.0;
    let pixel_height = (fovrad / 2.0).tan();
    let pixel_width = pixel_height * (self.config.width as f32 / self.config.height as f32);

    let img_width_by_2 = self.config.width as f32 / 2.0;
    let img_height_by_2 = self.config.height as f32 / 2.0;

    // --- Parallel Rendering ---
    // We iterate over the buffer in chunks exactly equal to the image width.
    // Each chunk represents one horizontal scanline.
    image_data.par_chunks_mut(width)
        .enumerate() // This gives us the row index (y)
        .for_each(|(y, row)| {
            
            // Optimization: 'b' depends only on 'y', so we calculate it once per row
            // instead of once per pixel.
            let b = (pixel_height * (img_height_by_2 - (y as f32 + 0.5))) / img_height_by_2;

            for (x, pixel) in row.iter_mut().enumerate() {
                // 'a' depends on 'x'
                let a = (pixel_width * ((x as f32 + 0.5) - img_width_by_2)) / img_width_by_2;
                
                let d = (normal_to_plane * a + v * b + camera_vector).normalize();
                
                // Since `self` is shared (read-only), we can call this safely 
                // as long as find_color doesn't mutate `self`.
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

    fn find_color(&self, origin: glam::Vec3, direction: glam::Vec3) -> u32 {
        let ray: Ray = Ray { origin, direction };
        let ambient = self.config.ambient;
        let closest_intersection = self
            .config
            .get_scene_objects()
            .iter()
            .filter_map(|object| object.intersect(&ray))
            .min_by(|a, b| {
                a.distance
                    .partial_cmp(&b.distance)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        if let Some(intersection) = closest_intersection {
            let mut final_color = ambient;
            for light in self.config.get_lights() {
                // shadow ray
                let light_dir = match light {
                    Point { position, .. } => (*position - intersection.point).normalize(),
                    Directional { direction, .. } => (-*direction).normalize(),
                };
                let shadow_ray = Ray {
                    origin: intersection.point + intersection.normal * 0.001,
                    direction: light_dir,
                };
                let in_shadow = self
                    .config
                    .get_scene_objects()
                    .iter()
                    .filter_map(|object| object.intersect(&shadow_ray))
                    .any(|shadow_intersection| match light {
                        Point { position, .. } => {
                            shadow_intersection.distance < (*position - intersection.point).length()
                        }
                        Directional { .. } => true,
                    });
                if in_shadow {
                    continue;
                }
                // blinn-phong shading
                let light_color = light.color();
                let n_dot_l = intersection.normal.dot(light_dir).max(0.0);
                let diffuse = intersection.diffuse_color * n_dot_l;
                let view_dir = (self.config.camera.position - intersection.point).normalize();
                let half_vector = (light_dir + view_dir).normalize();
                let n_dot_h = intersection.normal.dot(half_vector).max(0.0);
                let specular = intersection.specular_color * n_dot_h.powf(intersection.shininess);
                final_color += (diffuse + specular) * light_color;
            }
            let r = (final_color.x * 255.0).min(255.0) as u32;
            let g = (final_color.y * 255.0).min(255.0) as u32;
            let b = (final_color.z * 255.0).min(255.0) as u32;
            (255 << 24) | (r << 16) | (g << 8) | b
        } else {
            255 << 24
        }
    }
}

#[cfg(test)]
mod tests {
    // load test_file/jalon3/tp31.test and tp31.png and run the raytracer
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
