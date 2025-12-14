use glam::Vec3;
use crate::raytracer::config::shapes::sphere::Sphere;
use crate::raytracer::config::shapes::sphere::Intersectable;
pub enum Shape {
    Sphere(Sphere),
    // Plane(Plane),
    // Triangle(Triangle),
}

pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Shape {
    // Helper method to dispatch the call
    pub fn intersect(&self, ray: &Ray) -> Option<f32> {
        match self {
            Shape::Sphere(s) => (s as &dyn Intersectable).intersect(ray),
            // Shape::Plane(p) => p.intersect(ray),
            // Shape::Triangle(t) => t.intersect(ray),
        }
    }
}