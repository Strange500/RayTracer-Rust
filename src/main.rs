// Image and all functions are in imgcomparator module
mod imgcomparator;
mod raytracer;

use raytracer::ParsedConfigState;
// Assuming Config looks like: struct Config { width: u32, ... }

fn main() {
    let mut parsed_config = ParsedConfigState::new();
    let config = parsed_config.load_config_file("final_avec_bonus.scene").expect("Failed to load configuration");
    println!("Configuration loaded successfully.");
    let ray_tracer = raytracer::RayTracer::new(config);
    println!("Starting rendering...");
    let image = ray_tracer.render();
    match image {
        Ok(img) => {
            imgcomparator::save_image(&img, ray_tracer.get_output_path())
                .expect("Failed to save image");
            println!("Image rendered and saved to output.png");
        }
        Err(e) => {
            eprintln!("Error during rendering: {e}");
        }
    }
}
