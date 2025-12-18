// Image and all functions are in imgcomparator module
mod imgcomparator;
mod raytracer;

use raytracer::ParsedConfigState;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let use_gpu = args.contains(&"--gpu".to_string());
    
    let mut parsed_config = ParsedConfigState::new();
    let config = parsed_config.load_config_file("final_avec_bonus.scene").expect("Failed to load configuration");
    println!("Configuration loaded successfully.");
    let ray_tracer = raytracer::RayTracer::new(config);
    
    if use_gpu {
        println!("Starting GPU rendering...");
        let start_time = std::time::Instant::now();
        let image = ray_tracer.render_gpu();
        let duration = start_time.elapsed();
        println!("GPU rendering completed in: {:?}", duration);
        match image {
            Ok(img) => {
                imgcomparator::save_image(&img, ray_tracer.get_output_path())
                    .expect("Failed to save image");
                println!("Image rendered with GPU and saved to {}", ray_tracer.get_output_path());
            }
            Err(e) => {
                eprintln!("Error during GPU rendering: {e}");
                eprintln!("Falling back to CPU rendering...");
                cpu_render(&ray_tracer);
            }
        }
    } else {
        cpu_render(&ray_tracer);
    }
}

fn cpu_render(ray_tracer: &raytracer::RayTracer) {
    println!("Starting CPU rendering...");
    let start_time = std::time::Instant::now();
    let image = ray_tracer.render();
    let duration = start_time.elapsed();
    println!("CPU rendering completed in: {:?}", duration);
    match image {
        Ok(img) => {
            imgcomparator::save_image(&img, ray_tracer.get_output_path())
                .expect("Failed to save image");
            println!("Image rendered with CPU and saved to {}", ray_tracer.get_output_path());
        }
        Err(e) => {
            eprintln!("Error during rendering: {e}");
        }
    }
}
