// WebGPU Shading Language (WGSL) shader for ray tracing
// This compute shader implements basic ray tracing with sphere intersection,
// lighting (diffuse and specular), shadows, and reflections.

// Scene parameters
struct SceneParams {
    width: u32,
    height: u32,
    max_depth: u32,
    sphere_count: u32,
    light_count: u32,
    _padding: vec3<u32>,
    ambient: vec3<f32>,
    _padding2: f32,
}

// Camera parameters
struct Camera {
    position: vec3<f32>,
    _padding1: f32,
    direction: vec3<f32>,
    _padding2: f32,
    up: vec3<f32>,
    fov: f32,
}

// Sphere definition
struct Sphere {
    center: vec3<f32>,
    radius: f32,
    diffuse: vec3<f32>,
    _padding1: f32,
    specular: vec3<f32>,
    shininess: f32,
}

// Light definition
struct Light {
    position_or_direction: vec3<f32>,
    light_type: u32, // 0 = point, 1 = directional
    color: vec3<f32>,
    _padding: u32,
}

// Ray structure
struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
}

// Intersection result
struct Intersection {
    hit: bool,
    distance: f32,
    point: vec3<f32>,
    normal: vec3<f32>,
    diffuse: vec3<f32>,
    specular: vec3<f32>,
    shininess: f32,
}

// Bind groups
@group(0) @binding(0) var<uniform> scene: SceneParams;
@group(0) @binding(1) var<uniform> camera: Camera;
@group(0) @binding(2) var<storage, read> spheres: array<Sphere>;
@group(0) @binding(3) var<storage, read> lights: array<Light>;
@group(0) @binding(4) var<storage, read_write> output: array<u32>;

// Constants
const EPSILON: f32 = 1e-6;
const PI: f32 = 3.14159265359;

// Sphere-ray intersection
fn intersect_sphere(ray: Ray, sphere: Sphere) -> Intersection {
    var result: Intersection;
    result.hit = false;
    result.distance = 1e10;
    
    let oc = ray.origin - sphere.center;
    let half_b = dot(oc, ray.direction);
    let c = dot(oc, oc) - sphere.radius * sphere.radius;
    let discriminant = half_b * half_b - c;
    
    if (discriminant < 0.0) {
        return result;
    }
    
    let t = -half_b - sqrt(discriminant);
    if (t < EPSILON) {
        return result;
    }
    
    result.hit = true;
    result.distance = t;
    result.point = ray.origin + ray.direction * t;
    result.normal = normalize(result.point - sphere.center);
    result.diffuse = sphere.diffuse;
    result.specular = sphere.specular;
    result.shininess = sphere.shininess;
    
    return result;
}

// Find closest intersection with any sphere
fn find_closest_intersection(ray: Ray) -> Intersection {
    var closest: Intersection;
    closest.hit = false;
    closest.distance = 1e10;
    
    for (var i = 0u; i < scene.sphere_count; i = i + 1u) {
        let hit = intersect_sphere(ray, spheres[i]);
        if (hit.hit && hit.distance < closest.distance) {
            closest = hit;
        }
    }
    
    return closest;
}

// Check if point is in shadow
fn is_in_shadow(point: vec3<f32>, light_dir: vec3<f32>, max_distance: f32) -> bool {
    let shadow_ray = Ray(point + light_dir * EPSILON, light_dir);
    
    for (var i = 0u; i < scene.sphere_count; i = i + 1u) {
        let hit = intersect_sphere(shadow_ray, spheres[i]);
        if (hit.hit && hit.distance < max_distance) {
            return true;
        }
    }
    
    return false;
}

// Calculate color with lighting
fn calculate_lighting(intersection: Intersection, view_dir: vec3<f32>) -> vec3<f32> {
    var color = scene.ambient;
    
    for (var i = 0u; i < scene.light_count; i = i + 1u) {
        let light = lights[i];
        var light_dir: vec3<f32>;
        var max_distance: f32 = 1e10;
        
        if (light.light_type == 0u) {
            // Point light
            let to_light = light.position_or_direction - intersection.point;
            max_distance = length(to_light);
            light_dir = normalize(to_light);
        } else {
            // Directional light
            light_dir = normalize(light.position_or_direction);
        }
        
        // Check for shadows
        if (is_in_shadow(intersection.point, light_dir, max_distance)) {
            continue;
        }
        
        // Diffuse lighting
        let n_dot_l = max(dot(intersection.normal, light_dir), 0.0);
        let diffuse = intersection.diffuse * n_dot_l;
        
        // Specular lighting (Blinn-Phong)
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
        
        let specular = intersection.specular * specular_factor;
        
        color = color + (diffuse + specular) * light.color;
    }
    
    return color;
}

// Trace ray with reflections
fn trace_ray(initial_ray: Ray, max_depth: u32) -> vec3<f32> {
    var color = vec3<f32>(0.0, 0.0, 0.0);
    var ray = initial_ray;
    var reflection_factor = vec3<f32>(1.0, 1.0, 1.0);
    
    for (var depth = 0u; depth < max_depth; depth = depth + 1u) {
        let intersection = find_closest_intersection(ray);
        
        if (!intersection.hit) {
            break;
        }
        
        let view_dir = -ray.direction;
        let lighting = calculate_lighting(intersection, view_dir);
        
        color = color + reflection_factor * lighting;
        
        // Check if surface is reflective
        let is_reflective = intersection.specular.x > 0.0 || 
                           intersection.specular.y > 0.0 || 
                           intersection.specular.z > 0.0;
        
        if (!is_reflective || depth + 1u >= max_depth) {
            break;
        }
        
        // Calculate reflection ray
        let reflect_dir = reflect(ray.direction, intersection.normal);
        ray = Ray(intersection.point + intersection.normal * EPSILON, reflect_dir);
        reflection_factor = reflection_factor * intersection.specular;
    }
    
    return color;
}

// Pack RGB color to u32 (ARGB format)
fn pack_color(color: vec3<f32>) -> u32 {
    let r = u32(clamp(color.x, 0.0, 1.0) * 255.0);
    let g = u32(clamp(color.y, 0.0, 1.0) * 255.0);
    let b = u32(clamp(color.z, 0.0, 1.0) * 255.0);
    return (255u << 24u) | (r << 16u) | (g << 8u) | b;
}

// Main compute shader entry point
@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
    
    // Check bounds
    if (x >= scene.width || y >= scene.height) {
        return;
    }
    
    // Calculate ray direction
    let fov_rad = camera.fov * PI / 180.0;
    let pixel_height = tan(fov_rad / 2.0);
    let pixel_width = pixel_height * (f32(scene.width) / f32(scene.height));
    
    let img_width_by_2 = f32(scene.width) / 2.0;
    let img_height_by_2 = f32(scene.height) / 2.0;
    
    let a = (pixel_width * (f32(x) + 0.5 - img_width_by_2)) / img_width_by_2;
    let b = (pixel_height * (img_height_by_2 - (f32(y) + 0.5))) / img_height_by_2;
    
    // Calculate camera basis vectors
    let camera_forward = normalize(camera.direction);
    let camera_right = normalize(cross(camera_forward, camera.up));
    let camera_up = cross(camera_right, camera_forward);
    
    let ray_direction = normalize(camera_right * a + camera_up * b + camera_forward);
    
    // Trace ray
    let ray = Ray(camera.position, ray_direction);
    let color = trace_ray(ray, scene.max_depth);
    
    // Write pixel
    let pixel_index = y * scene.width + x;
    output[pixel_index] = pack_color(color);
}
