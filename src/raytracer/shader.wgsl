// Ray Tracing Compute Shader in WGSL

struct Camera {
    position: vec3<f32>,
    direction: vec3<f32>,
    up: vec3<f32>,
    fov: f32,
    width: u32,
    height: u32,
}

struct Light {
    position_or_direction: vec3<f32>,
    color: vec3<f32>,
    light_type: u32, // 0 = point, 1 = directional
}

struct Sphere {
    center: vec3<f32>,
    radius: f32,
    diffuse_color: vec3<f32>,
    specular_color: vec3<f32>,
    shininess: f32,
}

struct Plane {
    point: vec3<f32>,
    normal: vec3<f32>,
    diffuse_color: vec3<f32>,
    specular_color: vec3<f32>,
    shininess: f32,
}

struct Triangle {
    v0: vec3<f32>,
    v1: vec3<f32>,
    v2: vec3<f32>,
    diffuse_color: vec3<f32>,
    specular_color: vec3<f32>,
    shininess: f32,
}

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
}

struct Intersection {
    distance: f32,
    normal: vec3<f32>,
    point: vec3<f32>,
    diffuse_color: vec3<f32>,
    specular_color: vec3<f32>,
    shininess: f32,
    hit: bool,
    is_back_face: bool,
}

struct SceneData {
    ambient: vec3<f32>,
    maxdepth: u32,
    num_spheres: u32,
    num_planes: u32,
    num_triangles: u32,
    num_lights: u32,
}

@group(0) @binding(0) var<uniform> camera: Camera;
@group(0) @binding(1) var<uniform> scene: SceneData;
@group(0) @binding(2) var<storage, read> spheres: array<Sphere>;
@group(0) @binding(3) var<storage, read> planes: array<Plane>;
@group(0) @binding(4) var<storage, read> triangles: array<Triangle>;
@group(0) @binding(5) var<storage, read> lights: array<Light>;
@group(0) @binding(6) var<storage, read_write> output: array<u32>;

fn intersect_sphere(ray: Ray, sphere: Sphere) -> Intersection {
    var result: Intersection;
    result.hit = false;
    
    let oc = ray.origin - sphere.center;
    let half_b = dot(oc, ray.direction);
    let c = dot(oc, oc) - sphere.radius * sphere.radius;
    let discriminant = half_b * half_b - c;
    
    if (discriminant < 0.0) {
        return result;
    }
    
    let t = -half_b - sqrt(discriminant);
    if (t < 0.0) {
        return result;
    }
    
    result.distance = t;
    result.point = ray.origin + ray.direction * t;
    result.normal = normalize(result.point - sphere.center);
    result.diffuse_color = sphere.diffuse_color;
    result.specular_color = sphere.specular_color;
    result.shininess = sphere.shininess;
    result.hit = true;
    result.is_back_face = false;
    
    return result;
}

fn intersect_plane(ray: Ray, plane: Plane) -> Intersection {
    var result: Intersection;
    result.hit = false;
    
    let denom = dot(plane.normal, ray.direction);
    if (abs(denom) < 1e-6) {
        return result;
    }
    
    let t = dot(plane.point - ray.origin, plane.normal) / denom;
    if (t < 0.0) {
        return result;
    }
    
    result.distance = t;
    result.point = ray.origin + ray.direction * t;
    result.normal = plane.normal;
    result.diffuse_color = plane.diffuse_color;
    result.specular_color = plane.specular_color;
    result.shininess = plane.shininess;
    result.hit = true;
    result.is_back_face = false;
    
    return result;
}

fn intersect_triangle(ray: Ray, triangle: Triangle) -> Intersection {
    var result: Intersection;
    result.hit = false;
    
    let edge1 = triangle.v1 - triangle.v0;
    let edge2 = triangle.v2 - triangle.v0;
    let h = cross(ray.direction, edge2);
    let a = dot(edge1, h);
    
    if (abs(a) < 1e-6) {
        return result;
    }
    
    let f = 1.0 / a;
    let s = ray.origin - triangle.v0;
    let u = f * dot(s, h);
    
    if (u < 0.0 || u > 1.0) {
        return result;
    }
    
    let q = cross(s, edge1);
    let v = f * dot(ray.direction, q);
    
    if (v < 0.0 || u + v > 1.0) {
        return result;
    }
    
    let t = f * dot(edge2, q);
    if (t < 0.0) {
        return result;
    }
    
    result.distance = t;
    result.point = ray.origin + ray.direction * t;
    result.normal = normalize(cross(edge1, edge2));
    result.diffuse_color = triangle.diffuse_color;
    result.specular_color = triangle.specular_color;
    result.shininess = triangle.shininess;
    result.hit = true;
    result.is_back_face = dot(result.normal, ray.direction) > 0.0;
    
    return result;
}

fn find_closest_intersection(ray: Ray) -> Intersection {
    var closest: Intersection;
    closest.hit = false;
    closest.distance = 1e10;
    
    // Check spheres
    for (var i = 0u; i < scene.num_spheres; i++) {
        let intersection = intersect_sphere(ray, spheres[i]);
        if (intersection.hit && intersection.distance < closest.distance) {
            closest = intersection;
        }
    }
    
    // Check planes
    for (var i = 0u; i < scene.num_planes; i++) {
        let intersection = intersect_plane(ray, planes[i]);
        if (intersection.hit && intersection.distance < closest.distance) {
            closest = intersection;
        }
    }
    
    // Check triangles
    for (var i = 0u; i < scene.num_triangles; i++) {
        let intersection = intersect_triangle(ray, triangles[i]);
        if (intersection.hit && intersection.distance < closest.distance) {
            closest = intersection;
        }
    }
    
    return closest;
}

fn is_in_shadow(point: vec3<f32>, light_dir: vec3<f32>, light_type: u32, light_pos: vec3<f32>, intersection: Intersection) -> bool {
    let shadow_ray = Ray(point + intersection.normal * 1e-6, light_dir);
    
    // Check spheres
    for (var i = 0u; i < scene.num_spheres; i++) {
        let shadow_intersection = intersect_sphere(shadow_ray, spheres[i]);
        if (shadow_intersection.hit && shadow_intersection.distance > 1e-6) {
            if (light_type == 1u) {
                return true; // directional light
            }
            if (shadow_intersection.distance < length(light_pos - point)) {
                return true; // point light
            }
        }
    }
    
    // Check planes
    for (var i = 0u; i < scene.num_planes; i++) {
        let shadow_intersection = intersect_plane(shadow_ray, planes[i]);
        if (shadow_intersection.hit && shadow_intersection.distance > 1e-6) {
            if (light_type == 1u) {
                return true;
            }
            if (shadow_intersection.distance < length(light_pos - point)) {
                return true;
            }
        }
    }
    
    // Check triangles
    for (var i = 0u; i < scene.num_triangles; i++) {
        let shadow_intersection = intersect_triangle(shadow_ray, triangles[i]);
        if (shadow_intersection.hit && shadow_intersection.distance > 1e-6) {
            if (intersection.is_back_face && shadow_intersection.is_back_face) {
                continue;
            }
            if (light_type == 1u) {
                return true;
            }
            if (shadow_intersection.distance < length(light_pos - point)) {
                return true;
            }
        }
    }
    
    return false;
}

fn trace_ray(initial_ray: Ray) -> vec3<f32> {
    var color = vec3<f32>(0.0);
    var ray = initial_ray;
    var reflectivity = vec3<f32>(1.0);
    
    // Iterative ray tracing (instead of recursion)
    for (var depth = 0u; depth < scene.maxdepth; depth++) {
        let intersection = find_closest_intersection(ray);
        
        if (!intersection.hit) {
            break;
        }
        
        // Accumulate light contributions
        var light_accumulator = vec3<f32>(0.0);
        
        for (var i = 0u; i < scene.num_lights; i++) {
            let light = lights[i];
            var light_dir: vec3<f32>;
            
            if (light.light_type == 0u) {
                // Point light
                light_dir = normalize(light.position_or_direction - intersection.point);
            } else {
                // Directional light
                light_dir = light.position_or_direction;
            }
            
            if (!is_in_shadow(intersection.point, light_dir, light.light_type, light.position_or_direction, intersection)) {
                let n_dot_l = max(dot(intersection.normal, light_dir), 0.0);
                let diffuse = intersection.diffuse_color * n_dot_l;
                
                let view_dir = -ray.direction;
                let half_vector = normalize(light_dir + view_dir);
                let n_dot_h = max(dot(intersection.normal, half_vector), 0.0);
                
                var specular_factor: f32;
                if (intersection.shininess == 1.0) {
                    specular_factor = n_dot_h;
                } else if (intersection.shininess == 0.0) {
                    if (n_dot_l > 0.0) {
                        specular_factor = n_dot_h;
                    } else {
                        specular_factor = 0.0;
                    }
                } else {
                    if (n_dot_l > 0.0) {
                        specular_factor = pow(n_dot_h, intersection.shininess);
                    } else {
                        specular_factor = 0.0;
                    }
                }
                
                let specular = intersection.specular_color * specular_factor;
                light_accumulator += (diffuse + specular) * light.color;
            }
        }
        
        color += reflectivity * (light_accumulator + scene.ambient);
        
        // Check if we should continue with reflection
        let is_reflective = intersection.specular_color.x > 0.0 || 
                           intersection.specular_color.y > 0.0 || 
                           intersection.specular_color.z > 0.0;
        
        if (!is_reflective || depth + 1u >= scene.maxdepth) {
            break;
        }
        
        // Set up reflection ray
        let reflect_dir = ray.direction - 2.0 * dot(ray.direction, intersection.normal) * intersection.normal;
        ray.origin = intersection.point + intersection.normal * 1e-6;
        ray.direction = reflect_dir;
        reflectivity *= intersection.specular_color;
    }
    
    return color;
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
    
    if (x >= camera.width || y >= camera.height) {
        return;
    }
    
    // Calculate ray direction
    let camera_vector = normalize(camera.direction);
    let normal_to_plane = normalize(cross(camera_vector, camera.up));
    let v = normalize(cross(normal_to_plane, camera_vector));
    
    let fovrad = camera.fov * 3.14159265359 / 180.0;
    let pixel_height = tan(fovrad / 2.0);
    let pixel_width = pixel_height * (f32(camera.width) / f32(camera.height));
    
    let img_width_by_2 = f32(camera.width) / 2.0;
    let img_height_by_2 = f32(camera.height) / 2.0;
    
    let a = (pixel_width * (f32(x) + 0.5 - img_width_by_2)) / img_width_by_2;
    let b = (pixel_height * (img_height_by_2 - f32(y) - 0.5)) / img_height_by_2;
    
    let d = normalize(normal_to_plane * a + v * b + camera_vector);
    
    let ray = Ray(camera.position, d);
    let color_vec = trace_ray(ray);
    
    // Convert to RGBA
    let r = u32(clamp(color_vec.x, 0.0, 1.0) * 255.0);
    let g = u32(clamp(color_vec.y, 0.0, 1.0) * 255.0);
    let b = u32(clamp(color_vec.z, 0.0, 1.0) * 255.0);
    let color_u32 = (255u << 24u) | (r << 16u) | (g << 8u) | b;
    
    let index = y * camera.width + x;
    output[index] = color_u32;
}
