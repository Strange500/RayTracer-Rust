use glam::Vec3;

// 1. Added derive Copy and Clone so we can store the Shape in the Intersection struct easily.
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
    pub direction: Vec3,
}

pub struct Intersection {
    pub distance: f32,
    pub normal: Vec3,
    pub point: Vec3,
    pub shape: Shape,
}

impl Shape {
    pub fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        match self {
            // 2. Fixed syntax: Use { .. } to match a struct-variant without binding fields
            Shape::Sphere { .. } => intersect_sphere(ray, self),
        }
    }

    pub fn get_diffuse_color(&self) -> Vec3 {
        match self {
            // 3. Use .. to ignore fields we don't need here
            Shape::Sphere { diffuse_color, .. } => *diffuse_color,
        }
    }
}

fn intersect_sphere(ray: &Ray, sphere: &Shape) -> Option<Intersection> {
    // 4. Use if let to destruct the sphere fields we need for math
    if let Shape::Sphere { center, radius, .. } = sphere {
        let oc = ray.origin - *center;
        let a = ray.direction.dot(ray.direction);
        let b = 2.0 * oc.dot(ray.direction);
        let c = oc.dot(oc) - radius * radius;
        let discriminant = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            None
        } else {
            // Calculate nearest intersection
            let t = (-b - discriminant.sqrt()) / (2.0 * a);

            if t < 0.0 {
                return None;
            }

            let point = ray.origin + ray.direction * t;
            let normal = (point - *center).normalize();

            Some(Intersection {
                distance: t,
                normal,
                point,
                // 5. Since we derived Clone/Copy, we can dereference or clone here
                shape: *sphere,
            })
        }
    } else {
        None
    }
}
