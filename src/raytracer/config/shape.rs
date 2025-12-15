use glam::Vec3;
pub enum Shape {
    Sphere {
        center: Vec3,
        radius: f32,
        diffuse_color: Vec3,
        specular_color: Vec3,
        shininess: f32,
    },
    // Plane(Plane),
    // Triangle(Triangle),
}

pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

pub struct Intersection {
    pub distance: f32,
    pub normal: glam::Vec3,
    pub point: glam::Vec3,
    pub shape: Shape,
}

impl Shape {
    // Helper method to dispatch the call
    pub fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        match self {
            Shape::Sphere { center, radius, diffuse_color, specular_color, shininess } => 
                intersect_sphere(ray, *center, *radius, *diffuse_color, *specular_color, *shininess),
        }
    }
}

fn intersect_sphere(ray: &Ray, center: Vec3, radius: f32, diffuse_color: Vec3, specular_color: Vec3, shininess: f32) -> Option<Intersection> {
    let oc = ray.origin - center;
    let a = ray.direction.dot(ray.direction);
    let b = 2.0 * oc.dot(ray.direction);
    let c = oc.dot(oc) - radius * radius;
    let discriminant = b * b - 4.0 * a * c;

    if discriminant < 0.0 {
        None
    } else {
        let t = (-b - discriminant.sqrt()) / (2.0 * a);
        if t < 0.0 {
            return None;
        }
        let point = ray.origin + ray.direction * t;
        let normal = (point - center).normalize();
        Some(Intersection {
            distance: t,
            normal,
            point,
            shape: Shape::Sphere {
                center,
                radius,
                diffuse_color,
                specular_color,
                shininess,
            },
        })
    }
}
