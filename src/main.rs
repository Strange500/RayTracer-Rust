// Image and all functions are in imgcomparator module
mod imgcomparator;
mod raytracer;

use raytracer::load_config_file;
// Assuming Config looks like: struct Config { width: u32, ... }

fn main() {
    let config = load_config_file("tp31.test").expect("Failed to load configuration");
    let ray_tracer = raytracer::RayTracer::new(config);
    ray_tracer.main();
}