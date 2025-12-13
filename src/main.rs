// Image and all functions are in imgcomparator module
mod imgcomparator;
mod raytracer;

use raytracer::load_config_file;
// Assuming Config looks like: struct Config { width: u32, ... }

fn main() {
    println!("RayTracer Rust - Image Comparator");
    
    // We don't assign to 'result' because the arms do different things (one fails, one succeeds)
    match load_config_file("test.scene") {
        Ok(config) => {
            // Logic when successful
            config.print_summary();
        }
        Err(e) => {
            // Logic when failed
            eprintln!("Failed to load configuration: {}", e);
        }
    }
}