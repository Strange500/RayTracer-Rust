use crate::raytracer::config::camera::Camera;
use crate::raytracer::config::light::Light;
use crate::raytracer::config::shape::Shape;
use crate::raytracer::config::shapes::Sphere::Sphere;

use glam::Vec3;
use std::fs::File;
use std::io::{self, BufRead};
const COMMENT_CHAR: char = '#';

/// Configuration for the raytracer scene
///
/// This struct uses Rust's visibility system:
/// - `pub` fields like `width`, `height` can be accessed directly from outside
/// - Private fields like `scene_objects`, `lights` are encapsulated and only accessible via getter methods
/// 
/// This is a common Rust pattern for data hiding and encapsulation, ensuring
/// that internal state can only be modified through controlled means.
pub struct Config {
    pub width: u32,
    pub height: u32,
    pub output_file: String,
    pub camera: Camera,
    pub ambient: Vec3,
    /// Private: use get_scene_objects() to access
    scene_objects: Vec<Shape>,
    /// Private: use get_lights() to access
    lights: Vec<Light>,
}

impl Config {
    /// Returns a reference to the list of scene objects
    ///
    /// This is a "getter" method that provides read-only access to private data.
    /// The return type `&[Shape]` is a slice reference - it's borrowed data that
    /// the caller can read but not modify or take ownership of.
    pub fn get_scene_objects(&self) -> &[Shape] {
        &self.scene_objects
    }

    /// Returns a reference to the list of lights
    ///
    /// Similar to get_scene_objects, this provides read-only access via borrowing.
    /// The `&self` parameter means this method borrows the Config instance immutably.
    pub fn get_lights(&self) -> &[Light] {
        &self.lights
    }

    pub fn print_summary(&self) {
        println!("Configuration Summary:");
        println!("Image Size: {}x{}", self.width, self.height);
        println!("Output File: {}", self.output_file);
        println!(
            "Camera Position: ({}, {}, {}), LookAt: ({}, {}, {}), Up: ({}, {}, {}), FOV: {}",
            self.camera.position.x,
            self.camera.position.y,
            self.camera.position.z,
            self.camera.look_at.x,
            self.camera.look_at.y,
            self.camera.look_at.z,
            self.camera.up.x,
            self.camera.up.y,
            self.camera.up.z,
            self.camera.fov
        );
        println!(
            "Ambient Light: R: {}, G: {}, B: {}",
            self.ambient.x, self.ambient.y, self.ambient.z
        );
        println!("Number of Scene Objects: {}", self.scene_objects.len());
        println!("Number of Lights: {}", self.lights.len());
    }
}

/// Loads and parses a scene configuration file
///
/// # Rust Concepts Demonstrated:
/// - **Result Type**: Returns `Result<Config, String>` - either Ok(config) or Err(message)
/// - **Error Propagation**: The `?` operator automatically returns errors up the call stack
/// - **map_err**: Converts one error type to another (io::Error -> String)
///
/// # Arguments
/// * `file_path` - Path to the .scene file to load
///
/// # Returns
/// * `Ok(Config)` - Successfully parsed configuration
/// * `Err(String)` - Error message describing what went wrong
pub fn load_config_file(file_path: &str) -> Result<Config, String> {
    let file = File::open(file_path).map_err(|e| e.to_string())?;
    let reader = io::BufReader::new(file);
    let mut config = Config {
        width: 800,
        height: 600,
        output_file: "output.png".to_string(),
        camera: Camera {
            position: Vec3::ZERO,
            look_at: Vec3::Z,
            up: Vec3::Y,
            fov: 60.0,
        },
        ambient: Vec3::splat(0.1),
        scene_objects: Vec::new(),
        lights: Vec::new(),
    };
    for line in reader.lines() {
        parse_line(&line.map_err(|e| e.to_string())?, &mut config)?;
    }
    Ok(config)
}

fn parse_line(line: &str, config: &mut Config) -> Result<(), String> {
    if line.trim().is_empty() || line.trim_start().starts_with(COMMENT_CHAR) {
        return Ok(());
    }
    let parts: Vec<&str> = line.split(' ').map(|s| s.trim()).collect();
    if parts.len() >= 2 {
        // store in param all parts after the first as the value
        let param = &line[parts[0].len()..].trim();
        match parts[0] {
            "size" => {
                let (width, height) = parse_size(param)?;
                config.width = width;
                config.height = height;
            }
            "output" => {
                let output_file = parse_output(param)?;
                config.output_file = output_file;
            }
            "camera" => {
                let camera = parse_camera(param)?;
                config.camera = camera;
            }
            "ambient" => {
                let ambient = parse_ambient(param)?;
                config.ambient = ambient;
            }
            "sphere" => {
                let sphere = parse_sphere(param)?;
                config.scene_objects.push(Shape::Sphere(sphere));
            }
            "point" => {
                let light = parse_light(param)?;
                config.lights.push(light);
            }
            // Material properties - not yet implemented but recognized
            // TODO: Implement material parsing and associate with subsequent sphere
            "diffuse" | "specular" | "shininess" => {
                // Silently ignore for now - these will be implemented when
                // we add material support to shapes
            }
            _ => {
                // Unknown configuration key - return error to catch typos
                return Err(format!(
                    "Unknown configuration key: '{}'. Valid keys are: size, camera, output, ambient, sphere, point",
                    parts[0]
                ));
            }
        }
    }
    Ok(())
}
fn parse_size(value: &str) -> Result<(u32, u32), String> {
    let dims: Vec<&str> = value.split(' ').collect();
    if dims.len() != 2 {
        return Err("Invalid size format".to_string());
    }
    let width = dims[0].parse::<u32>().map_err(|e| e.to_string())?;
    let height = dims[1].parse::<u32>().map_err(|e| e.to_string())?;

    if width == 0 || height == 0 {
        return Err("Width and height must be greater than zero".to_string());
    }

    Ok((width, height))
}

fn parse_camera(value: &str) -> Result<Camera, String> {
    let params: Vec<&str> = value.split(' ').collect();
    if params.len() != 10 {
        return Err("Invalid camera format".to_string());
    }
    let position = Vec3::new(
        params[0].parse::<f32>().map_err(|e| e.to_string())?,
        params[1].parse::<f32>().map_err(|e| e.to_string())?,
        params[2].parse::<f32>().map_err(|e| e.to_string())?,
    );
    let look_at = Vec3::new(
        params[3].parse::<f32>().map_err(|e| e.to_string())?,
        params[4].parse::<f32>().map_err(|e| e.to_string())?,
        params[5].parse::<f32>().map_err(|e| e.to_string())?,
    );
    let up = Vec3::new(
        params[6].parse::<f32>().map_err(|e| e.to_string())?,
        params[7].parse::<f32>().map_err(|e| e.to_string())?,
        params[8].parse::<f32>().map_err(|e| e.to_string())?,
    );
    let fov = params[9].parse::<f32>().map_err(|e| e.to_string())?;

    if fov < 1.0 || fov > 179.0 {
        return Err("Field of view (fov) must be between 1 and 179 degrees".to_string());
    }

    Ok(Camera {
        position,
        look_at,
        up,
        fov,
    })
}

fn parse_ambient(value: &str) -> Result<Vec3, String> {
    let comps: Vec<&str> = value.split(' ').collect();
    if comps.len() != 3 {
        return Err("Invalid ambient light format".to_string());
    }
    let r = comps[0].parse::<f32>().map_err(|e| e.to_string())?;
    let g = comps[1].parse::<f32>().map_err(|e| e.to_string())?;
    let b = comps[2].parse::<f32>().map_err(|e| e.to_string())?;

    check_rgb_values(r, g, b)?;

    Ok(Vec3::new(r, g, b))
}

fn parse_output(value: &str) -> Result<String, String> {
    let output_file = value.trim();
    if output_file.is_empty() {
        return Err("Output file name cannot be empty".to_string());
    }
    Ok(output_file.to_string())
}

fn check_rgb_values(r: f32, g: f32, b: f32) -> Result<(), String> {
    if !(0.0..=1.0).contains(&r) || !(0.0..=1.0).contains(&g) || !(0.0..=1.0).contains(&b) {
        return Err("RGB values must be between 0.0 and 1.0".to_string());
    }
    Ok(())
}

fn parse_sphere(_value: &str) -> Result<Sphere, String> {
    // position + radius
    let params: Vec<&str> = _value.split(' ').collect();
    if params.len() != 4 {
        return Err("Invalid sphere format".to_string());
    }
    let center = Vec3::new(
        params[0].parse::<f32>().map_err(|e| e.to_string())?,
        params[1].parse::<f32>().map_err(|e| e.to_string())?,
        params[2].parse::<f32>().map_err(|e| e.to_string())?,
    );
    let radius = params[3].parse::<f32>().map_err(|e| e.to_string())?;
    if radius <= 0.0 {
        return Err("Sphere radius must be greater than zero".to_string());
    }
    Ok(Sphere::new(center, radius))
}

/// Parses a point light from the scene file
/// Format: "x y z r g b" where xyz is position and rgb is intensity
fn parse_light(value: &str) -> Result<Light, String> {
    let params: Vec<&str> = value.split(' ').collect();
    if params.len() != 6 {
        return Err("Invalid light format. Expected: x y z r g b".to_string());
    }
    
    // Parse position (x, y, z)
    let position = Vec3::new(
        params[0].parse::<f32>().map_err(|e| format!("Invalid light x position: {}", e))?,
        params[1].parse::<f32>().map_err(|e| format!("Invalid light y position: {}", e))?,
        params[2].parse::<f32>().map_err(|e| format!("Invalid light z position: {}", e))?,
    );
    
    // Parse intensity (r, g, b)
    let r = params[3].parse::<f32>().map_err(|e| format!("Invalid light red intensity: {}", e))?;
    let g = params[4].parse::<f32>().map_err(|e| format!("Invalid light green intensity: {}", e))?;
    let b = params[5].parse::<f32>().map_err(|e| format!("Invalid light blue intensity: {}", e))?;
    
    // Validate RGB values are in valid range
    check_rgb_values(r, g, b)?;
    
    let intensity = Vec3::new(r, g, b);
    
    Ok(Light::new(position, intensity))
}

// test

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_size() {
        let (width, height) = parse_size("1920 1080").unwrap();
        assert_eq!(width, 1920);
        assert_eq!(height, 1080);
    }

    #[test]
    fn test_parse_size_invalid_format() {
        assert!(parse_size("1920").is_err());
        assert!(parse_size("1920 1080 extra").is_err());
    }

    #[test]
    fn test_parse_size_zero_dimensions() {
        assert!(parse_size("0 1080").is_err());
        assert!(parse_size("1920 0").is_err());
    }

    #[test]
    fn test_parse_camera() {
        let camera = parse_camera("0.0 0.0 150.0 0.0 0.0 5.0 0.0 1.0 0.0 60").unwrap();
        assert_eq!(camera.position, Vec3::new(0.0, 0.0, 150.0));
        assert_eq!(camera.look_at, Vec3::new(0.0, 0.0, 5.0));
        assert_eq!(camera.up, Vec3::new(0.0, 1.0, 0.0));
        assert_eq!(camera.fov, 60.0);
    }

    #[test]
    fn test_parse_camera_invalid_fov() {
        // FOV too low (0 is invalid, must be >= 1)
        assert!(parse_camera("0.0 0.0 150.0 0.0 0.0 5.0 0.0 1.0 0.0 0").is_err());
        // FOV too high (180 is invalid, must be <= 179)
        assert!(parse_camera("0.0 0.0 150.0 0.0 0.0 5.0 0.0 1.0 0.0 180").is_err());
        // Negative FOV
        assert!(parse_camera("0.0 0.0 150.0 0.0 0.0 5.0 0.0 1.0 0.0 -45").is_err());
    }

    #[test]
    fn test_parse_camera_valid_fov_boundaries() {
        // FOV at minimum boundary (1 degree)
        assert!(parse_camera("0.0 0.0 150.0 0.0 0.0 5.0 0.0 1.0 0.0 1").is_ok());
        // FOV at maximum boundary (179 degrees)
        assert!(parse_camera("0.0 0.0 150.0 0.0 0.0 5.0 0.0 1.0 0.0 179").is_ok());
    }

    #[test]
    fn test_parse_ambient() {
        let ambient = parse_ambient("0.2 0.3 0.4").unwrap();
        assert_eq!(ambient, Vec3::new(0.2, 0.3, 0.4));
    }

    #[test]
    fn test_parse_ambient_invalid_rgb() {
        // RGB values outside valid range [0.0, 1.0]
        assert!(parse_ambient("1.5 0.3 0.4").is_err());
        assert!(parse_ambient("0.2 -0.1 0.4").is_err());
    }

    #[test]
    fn test_parse_output() {
        let output_file = parse_output("final.png").unwrap();
        assert_eq!(output_file, "final.png");
    }

    #[test]
    fn test_parse_output_empty() {
        assert!(parse_output("").is_err());
        assert!(parse_output("   ").is_err());
    }

    #[test]
    fn test_parse_sphere() {
        let sphere = parse_sphere("0.0 0.0 5.0 2.5").unwrap();
        assert_eq!(sphere.center, Vec3::new(0.0, 0.0, 5.0));
        assert_eq!(sphere.radius, 2.5);
    }

    #[test]
    fn test_parse_sphere_invalid_radius() {
        // Radius must be positive
        assert!(parse_sphere("0.0 0.0 5.0 0.0").is_err());
        assert!(parse_sphere("0.0 0.0 5.0 -1.0").is_err());
    }

    #[test]
    fn test_parse_light() {
        let light = parse_light("10.0 20.0 30.0 0.9 0.8 0.7").unwrap();
        assert_eq!(light.position, Vec3::new(10.0, 20.0, 30.0));
        assert_eq!(light.intensity, Vec3::new(0.9, 0.8, 0.7));
    }

    #[test]
    fn test_parse_light_invalid_format() {
        // Too few parameters
        assert!(parse_light("10.0 20.0 30.0").is_err());
        // Too many parameters
        assert!(parse_light("10.0 20.0 30.0 0.9 0.8 0.7 extra").is_err());
    }

    #[test]
    fn test_parse_light_invalid_intensity() {
        // Intensity values outside valid range [0.0, 1.0]
        assert!(parse_light("10.0 20.0 30.0 1.5 0.8 0.7").is_err());
        assert!(parse_light("10.0 20.0 30.0 0.9 -0.1 0.7").is_err());
    }

    #[test]
    fn test_check_rgb_values() {
        // Valid RGB values
        assert!(check_rgb_values(0.0, 0.5, 1.0).is_ok());
        
        // Invalid RGB values
        assert!(check_rgb_values(-0.1, 0.5, 1.0).is_err());
        assert!(check_rgb_values(0.0, 1.5, 1.0).is_err());
        assert!(check_rgb_values(0.0, 0.5, -0.1).is_err());
    }

    #[test]
    fn test_load_config_file() {
        let config = load_config_file("test.scene").unwrap();
        
        // Verify basic settings
        assert_eq!(config.width, 1920);
        assert_eq!(config.height, 1080);
        assert_eq!(config.output_file, "final.png");
        
        // Verify camera
        assert_eq!(config.camera.fov, 45.0);
        
        // Verify ambient light
        assert_eq!(config.ambient, Vec3::new(0.2, 0.2, 0.2));
        
        // Verify lights were loaded
        assert_eq!(config.get_lights().len(), 1);
        let light = &config.get_lights()[0];
        assert_eq!(light.position, Vec3::new(0.0, 0.0, 40.0));
        assert_eq!(light.intensity, Vec3::new(0.9, 0.9, 0.9));
        
        // Verify spheres were loaded
        assert!(config.get_scene_objects().len() > 0);
    }
}
