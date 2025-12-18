use glam::Vec3;
use bvh::aabb::{Aabb, Bounded};
use bvh::bounding_hierarchy::BHShape;
use nalgebra::{Point3, Vector3};

#[derive(Clone, Copy, Debug)]
pub enum Shape {
    Sphere {
        center: Vec3,
        radius: f32,
        diffuse_color: Vec3,
        specular_color: Vec3,
        shininess: f32,
        node_index: usize,
    },
    Triangle {
        v0: Vec3,
        v1: Vec3,
        v2: Vec3,
        diffuse_color: Vec3,
        specular_color: Vec3,
        shininess: f32,
        node_index: usize,
    },
    Plane {
        point: Vec3,
        normal: Vec3,
        diffuse_color: Vec3,
        specular_color: Vec3,
        shininess: f32,
        node_index: usize,
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

    Some(Intersection {
        distance: t,
        normal: *normal,
        point: intersection_point,
        diffuse_color: *diffuse_color,
        specular_color: *specular_color,
        shininess: *shininess,
        is_back_face: false, 
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

// Helper functions to convert between glam (used by this project) and nalgebra (used by BVH crate)
fn vec3_to_point3(v: Vec3) -> Point3<f32> {
    Point3::new(v.x, v.y, v.z)
}

fn vec3_to_vector3(v: Vec3) -> Vector3<f32> {
    Vector3::new(v.x, v.y, v.z)
}

// ==================== BVH Trait Implementations ====================
// The following trait implementations enable the BVH (Bounding Volume Hierarchy)
// acceleration structure. Each shape must provide:
// 1. An axis-aligned bounding box (AABB) via the Bounded trait
// 2. A way to store/retrieve its position in the BVH tree via the BHShape trait

/// Size used for plane AABBs. Planes are infinite, so we use a very large but finite box.
const PLANE_AABB_SIZE: f32 = 1e10;

/// Implement Bounded trait to provide AABBs (Axis-Aligned Bounding Boxes) for each shape.
/// The BVH uses these AABBs to quickly determine which objects a ray might intersect.
impl Bounded<f32, 3> for Shape {
    fn aabb(&self) -> Aabb<f32, 3> {
        match self {
            Shape::Sphere { center, radius, .. } => {
                // Sphere AABB: cube centered at sphere center with side length 2*radius
                let half_size = Vector3::new(*radius, *radius, *radius);
                let center_point = vec3_to_point3(*center);
                let min = center_point - half_size;
                let max = center_point + half_size;
                Aabb::with_bounds(min, max)
            }
            Shape::Triangle { v0, v1, v2, .. } => {
                // Triangle AABB: minimum box that contains all three vertices
                let p0 = vec3_to_point3(*v0);
                let p1 = vec3_to_point3(*v1);
                let p2 = vec3_to_point3(*v2);
                
                let min = Point3::new(
                    p0.x.min(p1.x).min(p2.x),
                    p0.y.min(p1.y).min(p2.y),
                    p0.z.min(p1.z).min(p2.z),
                );
                let max = Point3::new(
                    p0.x.max(p1.x).max(p2.x),
                    p0.y.max(p1.y).max(p2.y),
                    p0.z.max(p1.z).max(p2.z),
                );
                
                Aabb::with_bounds(min, max)
            }
            Shape::Plane { .. } => {
                // Planes are infinite, so we create a very large AABB.
                // Note: Infinite primitives like planes don't benefit much from BVH,
                // but we need to provide an AABB for the trait implementation.
                let min = Point3::new(-PLANE_AABB_SIZE, -PLANE_AABB_SIZE, -PLANE_AABB_SIZE);
                let max = Point3::new(PLANE_AABB_SIZE, PLANE_AABB_SIZE, PLANE_AABB_SIZE);
                Aabb::with_bounds(min, max)
            }
        }
    }
}

/// Implement BHShape trait to allow shapes to store their position in the BVH tree.
/// The BVH library needs to track which tree node each shape belongs to.
impl BHShape<f32, 3> for Shape {
    fn set_bh_node_index(&mut self, index: usize) {
        match self {
            Shape::Sphere { node_index, .. } => *node_index = index,
            Shape::Triangle { node_index, .. } => *node_index = index,
            Shape::Plane { node_index, .. } => *node_index = index,
        }
    }

    fn bh_node_index(&self) -> usize {
        match self {
            Shape::Sphere { node_index, .. } => *node_index,
            Shape::Triangle { node_index, .. } => *node_index,
            Shape::Plane { node_index, .. } => *node_index,
        }
    }
}
