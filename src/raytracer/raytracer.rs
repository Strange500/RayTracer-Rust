use crate::raytracer::config::Config;
use crate::raytracer::config::Ray;
use crate::imgcomparator::Image;

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
                let d = (normal_to_plane * a + v * b - camera_vector).normalize();
                let color = self.find_color(self.config.camera.position, d);
                image_data[(y * self.config.width + x) as usize] = color;
            }
        }
        Ok(Image::new(self.config.width, self.config.height, image_data))
    }

    pub fn get_output_path(&self) -> &str {
        &self.config.output_file
    }

    fn find_color(&self, origin: glam::Vec3, direction: glam::Vec3) -> u32 {
        let ray : Ray = Ray { origin, direction };
        for object in self.config.get_scene_objects() {
            if let Some(t) = object.intersect(&ray) {
                return (255 << 24) | (255 << 16) | (0 << 8) | 0;
            }
        }

        // No hit -> return background (opaque black)
        (255 << 24) | (0 << 16) | (0 << 8) | 0
    }
}

#[cfg(test)]
mod tests {
    // load test_file/jalon3/tp31.test and tp31.png and run the raytracer
    use super::*;
    use crate::imgcomparator::{Image};
    use crate::imgcomparator::file_to_image;
    use crate::raytracer::load_config_file;
    #[test]
    fn test_raytracer_tp31() {
        let config = load_config_file("test_file/jalon3/tp31.test").expect("Failed to load configuration");
        let ray_tracer = RayTracer::new(config);
        let generated_image = ray_tracer.render().expect("Failed to render image");
        let expected_image = file_to_image("test_file/jalon3/tp31.png").expect("Failed to load expected image");
        let (diff, _img) = Image::compare(&generated_image, &expected_image).expect("Failed to compare images");
        assert_eq!(diff, 0, "Images differ! See tp31_diff.png for details.");
    }
}
