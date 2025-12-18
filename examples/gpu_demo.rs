//! Example program demonstrating GPU-accelerated ray tracing
//! 
//! This example shows how to use the GPU renderer and compares its performance
//! to the CPU renderer.

use raytracer_rust::raytracer::{ParsedConfigState, RayTracer};
use raytracer_rust::imgcomparator::save_image;

#[cfg(feature = "gpu")]
use raytracer_rust::raytracer::GPURenderer;

fn main() {
    println!("=== Ray Tracing Performance Comparison ===\n");

    // Load scene configuration
    let scene_file = "gpu_test.scene";
    let mut parsed_config = ParsedConfigState::new();
    let config = parsed_config
        .load_config_file(scene_file)
        .expect("Failed to load scene configuration");
    
    println!("Scene: {}", scene_file);
    println!("Resolution: {}x{}", config.width, config.height);
    println!("Objects: {} spheres", config.get_scene_objects().len());
    println!();

    // CPU Rendering
    println!("--- CPU Rendering (with BVH + Rayon) ---");
    let ray_tracer = RayTracer::new(config.clone());
    let cpu_start = std::time::Instant::now();
    let cpu_image = ray_tracer.render().expect("CPU rendering failed");
    let cpu_duration = cpu_start.elapsed();
    println!("Time: {:?}", cpu_duration);
    save_image(&cpu_image, "output_cpu.png").expect("Failed to save CPU image");
    println!("Saved: output_cpu.png");
    println!();

    // GPU Rendering
    #[cfg(feature = "gpu")]
    {
        println!("--- GPU Rendering (with wgpu compute shaders) ---");
        match GPURenderer::new() {
            Ok(gpu_renderer) => {
                let gpu_start = std::time::Instant::now();
                match gpu_renderer.render(&config) {
                    Ok(gpu_image) => {
                        let gpu_duration = gpu_start.elapsed();
                        println!("Time: {:?}", gpu_duration);
                        save_image(&gpu_image, "output_gpu.png")
                            .expect("Failed to save GPU image");
                        println!("Saved: output_gpu.png");
                        println!();

                        // Performance comparison
                        println!("--- Performance Comparison ---");
                        let speedup = cpu_duration.as_secs_f64() / gpu_duration.as_secs_f64();
                        println!("Speedup: {:.2}x", speedup);
                        if speedup > 1.0 {
                            println!("GPU is {:.2}x faster than CPU", speedup);
                        } else {
                            println!("CPU is {:.2}x faster than GPU", 1.0 / speedup);
                            println!("Note: For small scenes, CPU may be faster due to GPU overhead.");
                        }
                    }
                    Err(e) => {
                        eprintln!("GPU rendering failed: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to initialize GPU renderer: {}", e);
                eprintln!("GPU acceleration is not available on this system.");
            }
        }
    }

    #[cfg(not(feature = "gpu"))]
    {
        println!("--- GPU Rendering ---");
        println!("GPU support not compiled.");
        println!("To enable GPU rendering, build with: cargo build --features gpu");
    }

    println!("\n=== Comparison Complete ===");
}
