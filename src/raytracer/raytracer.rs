use crate::imgcomparator::Image;
use crate::raytracer::config::light::Light::{Directional, Point};
use crate::raytracer::config::shape::Shape::Sphere;
use crate::raytracer::config::Config;
use crate::raytracer::config::Ray;

pub struct RayTracer {
    config: Config,
}

impl RayTracer {
    pub fn new(config: Config) -> Self {
        RayTracer { config }
    }
    pub fn render(&self) -> Result<Image, String> {
        let mut image_data = vec![0u32; (self.config.width * self.config.height) as usize];
        let camera_vector = self.config.camera.direction().normalize();
        let normal_to_plane = camera_vector.cross(self.config.camera.up).normalize();
        let v = normal_to_plane.cross(camera_vector).normalize();
        let fovrad = self.config.camera.fov * std::f32::consts::PI / 180.0;
        let pixel_height = (fovrad / 2.0).tan();
        let pixel_width = pixel_height * (self.config.width as f32 / self.config.height as f32);

        let img_width_by_2 = self.config.width as f32 / 2.0;
        let img_height_by_2 = self.config.height as f32 / 2.0;
        for y in 0..self.config.height {
            for x in 0..self.config.width {
                // Compute normalized device coordinates
                let a = (pixel_width * ((x as f32 + 0.5) - img_width_by_2)) / img_width_by_2;
                let b = (pixel_height * (img_height_by_2 - (y as f32 + 0.5))) / img_height_by_2;
                let d = (normal_to_plane * a + v * b + camera_vector).normalize();
                let color = self.find_color(self.config.camera.position, d);
                image_data[(y * self.config.width + x) as usize] = color;
            }
        }
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
                // lambertian shading
                let light_dir = match light {
                    Point { position, .. } => (*position - intersection.point).normalize(),
                    Directional { direction, .. } => *direction,
                };
                let light_intensity = light.color();
                let lambertian = intersection.normal.dot(light_dir).max(0.0);
                let diffuse = &intersection.diffuse_color;
                final_color += diffuse * light_intensity * lambertian;
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
    fn test_raytracer_tp41dir() {
        test_file("test_file/jalon4/tp41-dir");
    }

    #[test]
    fn test_raytracer_tp41point() {
        test_file("test_file/jalon4/tp41-point");
    }

    #[test]
    fn test_raytracer_tp42dir() {
        test_file("test_file/jalon4/tp42-dir");
    }
    #[test]
    fn test_raytracer_tp42point() {
        test_file("test_file/jalon4/tp42-point");
    }

    #[test]
    fn test_raytracer_tp43() {
        test_file("test_file/jalon4/tp43");
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
