use glam::Vec3;

// Performance: Derive Copy for small structs to enable efficient passing by value
#[derive(Clone, Copy, Debug)]
pub enum Shape {
    Sphere {
        center: Vec3,
        radius: f32,
        diffuse_color: Vec3,
        specular_color: Vec3,
        shininess: f32,
    },
}

pub struct Ray {
    pub origin: Vec3,
    /// Direction vector. Must be normalized for correct intersection calculations.
    pub direction: Vec3,
}

pub struct Intersection {
    pub distance: f32,
    pub normal: Vec3,
    pub point: Vec3,
    pub diffuse_color: Vec3,
}

impl Shape {
    pub fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        match self {
            Shape::Sphere { .. } => intersect_sphere(ray, self),
        }
    }
}

fn intersect_sphere(ray: &Ray, sphere: &Shape) -> Option<Intersection> {
    let Shape::Sphere { center, radius, diffuse_color, .. } = sphere else {
        return None;
    };

    // IMPORTANT: This implementation assumes ray.direction is normalized (length = 1.0)
    // If ray.direction is not normalized, intersection calculations will be incorrect.
    debug_assert!((ray.direction.length() - 1.0).abs() < 0.001, "Ray direction must be normalized");
    
    let oc = ray.origin - *center;
    // Performance: Assume ray.direction is normalized (length = 1.0)
    // Using simplified quadratic formula: t = -half_b ± sqrt(discriminant)
    // where half_b = oc.dot(ray.direction), discriminant = half_b² - c
    let half_b = oc.dot(ray.direction);
    let c = oc.dot(oc) - radius * radius;
    let discriminant = half_b * half_b - c;

    if discriminant < 0.0 {
        None
    } else {
        // Calculate nearest intersection using simplified formula
        let t = -half_b - discriminant.sqrt();

        if t < 0.0 {
            return None;
        }

        let point = ray.origin + ray.direction * t;
        let normal = (point - *center).normalize();

        Some(Intersection {
            distance: t,
            normal,
            point,
            diffuse_color: *diffuse_color,
        })
    }
}
