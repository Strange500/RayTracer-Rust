use crate::raytracer::config::camera::Camera;
use crate::raytracer::config::light::Light;
use crate::raytracer::config::shape::Shape;
use crate::raytracer::config::shapes::Sphere::{Intersectable, Sphere};

use glam::Vec3;
use std::fs::File;
use std::io::{self, BufRead};
const COMMENT_CHAR: char = '#';

pub struct Config {
    pub width: u32,
    pub height: u32,
    pub output_file: String,
    pub camera: Camera,
    pub ambient: Vec3,
    scene_objects: Vec<Shape>,
    lights: Vec<Light>,
}

impl Config {
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
            _ => {
                // Return the error immediately to stop the function
                //return Err(format!("Unknown configuration key: {}", parts[0]));
                println!("Unknown configuration key: {}", parts[0]);
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
    fn test_parse_camera() {
        let camera = parse_camera("0.0 0.0 150.0 0.0 0.0 5.0 0.0 1.0 0.0 60").unwrap();
        assert_eq!(camera.position, Vec3::new(0.0, 0.0, 150.0));
        assert_eq!(camera.look_at, Vec3::new(0.0, 0.0, 5.0));
        assert_eq!(camera.up, Vec3::new(0.0, 1.0, 0.0));
        assert_eq!(camera.fov, 60.0);
    }

    #[test]
    fn test_parse_ambient() {
        let ambient = parse_ambient("0.2 0.3 0.4").unwrap();
        assert_eq!(ambient, Vec3::new(0.2, 0.3, 0.4));
    }

    #[test]
    fn test_parse_output() {
        let output_file = parse_output("final.png").unwrap();
        assert_eq!(output_file, "final.png");
    }
}
