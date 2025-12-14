use crate::raytracer::config::shape::Ray;
use glam::Vec3;
pub trait Intersectable {
    fn intersect(&self, ray: &Ray) -> Option<f32>;
}
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    diffuse_color: Vec3,
    specular_color: Vec3,
    shininess: f32,
}

impl Sphere {
    pub fn new(
        center: Vec3,
        radius: f32,
        diffuse_color: Vec3,
        specular_color: Vec3,
        shininess: f32,
    ) -> Self {
        Sphere {
            center,
            radius,
            diffuse_color,
            specular_color,
            shininess,
        }
    }
}

impl Intersectable for Sphere {
    fn intersect(&self, ray: &Ray) -> Option<f32> {
        let oc = ray.origin - self.center;
        let a = ray.direction.dot(ray.direction);
        let b = 2.0 * oc.dot(ray.direction);
        let c = oc.dot(oc) - self.radius * self.radius;
        let discriminant = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            None
        } else {
            Some((-b - discriminant.sqrt()) / (2.0 * a))
        }
    }
}
