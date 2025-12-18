use glam::Vec3;

#[derive(Clone, Copy, Debug)]
pub enum Shape {
    Sphere {
        center: Vec3,
        radius: f32,
        diffuse_color: Vec3,
        specular_color: Vec3,
        shininess: f32,
    },
    Triangle {
        v0: Vec3,
        v1: Vec3,
        v2: Vec3,
        diffuse_color: Vec3,
        specular_color: Vec3,
        shininess: f32,
    },
    Plane {
        point: Vec3,
        normal: Vec3,
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
    pub diffuse_color: Vec3,
    pub specular_color: Vec3,
    pub shininess: f32,
    pub is_back_face: bool,
}

impl Shape {
    pub fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        match self {
            Shape::Sphere { .. } => intersect_sphere(ray, self),
            Shape::Plane { .. } => intersect_plane(ray, self),
            Shape::Triangle { .. } => intersect_triangle(ray, self),
        }
    }
}

fn intersect_sphere(ray: &Ray, sphere: &Shape) -> Option<Intersection> {
    let Shape::Sphere {
        center,
        radius,
        diffuse_color,
        specular_color,
        shininess,
        ..
    } = sphere
    else {
        return None; // Not a sphere
    };

    let oc = ray.origin - *center;
    let half_b = oc.dot(ray.direction);
    let c = oc.dot(oc) - radius * radius;
    let discriminant = half_b * half_b - c;

    if discriminant < 0.0 {
        None
    } else {
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
            specular_color: *specular_color,
            shininess: *shininess,
            is_back_face: false,
        })
    }
}

fn intersect_plane(ray: &Ray, plane: &Shape) -> Option<Intersection> {
    let Shape::Plane {
        point,
        normal,
        diffuse_color,
        specular_color,
        shininess,
        ..
    } = plane
    else {
        return None;
    };

    let denom = normal.dot(ray.direction);
    if denom.abs() < 1e-6 {
        return None;
    }

    let t = (point - ray.origin).dot(*normal) / denom;
    if t < 0.0 {
        return None;
    }

    let intersection_point = ray.origin + ray.direction * t;
    
    // Check if we're hitting the plane from the back side
    let is_back_face = denom > 0.0;

    Some(Intersection {
        distance: t,
        normal: *normal,
        point: intersection_point,
        diffuse_color: *diffuse_color,
        specular_color: *specular_color,
        shininess: *shininess,
        is_back_face,
    })
}

fn intersect_triangle(ray: &Ray, triangle: &Shape) -> Option<Intersection> {
    let Shape::Triangle {
        v0,
        v1,
        v2,
        diffuse_color,
        specular_color,
        shininess,
        ..
    } = triangle
    else {
        return None;
    };

    let edge1 = *v1 - *v0;
    let edge2 = *v2 - *v0;
    let h = ray.direction.cross(edge2);
    let a = edge1.dot(h);

    if a.abs() < 1e-6 {
        return None;
    }

    let f = 1.0 / a;
    let s = ray.origin - *v0;
    let u = f * s.dot(h);

    if u < 0.0 || u > 1.0 {
        return None;
    }

    let q = s.cross(edge1);
    let v = f * ray.direction.dot(q);

    if v < 0.0 || u + v > 1.0 {
        return None;
    }

    let t = f * edge2.dot(q);
    if t < 0.0 {
        return None;
    }

    let intersection_point = ray.origin + ray.direction * t;
    let normal = edge1.cross(edge2).normalize();
    
    let is_back_face = normal.dot(ray.direction) > 0.0;

    Some(Intersection {
        distance: t,
        normal,
        point: intersection_point,
        diffuse_color: *diffuse_color,
        specular_color: *specular_color,
        shininess: *shininess,
        is_back_face,
    })
}
