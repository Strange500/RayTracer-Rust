use crate::raytracer::config::camera::Camera;
use crate::raytracer::config::light::Light;
use crate::raytracer::config::shape::Shape;

use glam::Vec3;
use std::fs::File;
use std::io::{self, BufRead};

const COMMENT_CHAR: char = '#';
const DEFAULT_DIFFUSE_COLOR: Vec3 = Vec3::ZERO;
const DEFAULT_SPECULAR_COLOR: Vec3 = Vec3::ZERO;
const DEFAULT_SHININESS: f32 = 0.0;

pub struct Config {
    pub width: u32,
    pub height: u32,
    pub output_file: String,
    pub camera: Camera,
    pub ambient: Vec3,
    pub maxdepth: u32,
    pub maxverts: u32,
    scene_objects: Vec<Shape>,
    lights: Vec<Light>,
}

impl Config {
    pub fn get_scene_objects(&self) -> &Vec<Shape> {
        &self.scene_objects
    }

    pub fn get_scene_objects_mut(&mut self) -> &mut Vec<Shape> {
        &mut self.scene_objects
    }

    pub fn get_lights(&self) -> &Vec<Light> {
        &self.lights
    }

    pub fn println_config(&self) {
        println!("Config:");
        println!(" Size: {}x{}", self.width, self.height);
        println!(" Output file: {}", self.output_file);
        println!(
            " Camera: position({:?}), look_at({:?}), up({:?}), fov({})",
            self.camera.position, self.camera.look_at, self.camera.up, self.camera.fov
        );
        println!(" Ambient light: {:?}", self.ambient);
        for (i, obj) in self.scene_objects.iter().enumerate() {
            match obj {
                Shape::Sphere {
                    center,
                    radius,
                    diffuse_color,
                    specular_color,
                    shininess,
                    ..
                } => {
                    println!(
                        " Object {}: Sphere - center({:?}), radius({}), diffuse_color({:?}), specular_color({:?}), shininess({})",
                        i, center, radius, diffuse_color, specular_color, shininess
                    );
                }
                Shape::Plane {
                    point,
                    normal,
                    diffuse_color,
                    specular_color,
                    shininess,
                    ..
                } => {
                    println!(
                        " Object {}: Plane - point({:?}), normal({:?}), diffuse_color({:?}), specular_color({:?}), shininess({})",
                        i, point, normal, diffuse_color, specular_color, shininess
                    );
                }
                Shape::Triangle {
                    v0,
                    v1,
                    v2,
                    diffuse_color,
                    specular_color,
                    shininess,
                    ..
                } => {
                    println!(
                        " Object {}: Triangle - v0({:?}), v1({:?}), v2({:?}), diffuse_color({:?}), specular_color({:?}), shininess({})",
                        i, v0, v1, v2, diffuse_color, specular_color, shininess
                    );
                }
            }
        }
        for (i, light) in self.lights.iter().enumerate() {
            match light {
                Light::Point { position, color } => {
                    println!(
                        " Light {}: Point - position({:?}), color({:?})",
                        i, position, color
                    );
                }
                Light::Directional { direction, color } => {
                    println!(
                        " Light {}: Directional - direction({:?}), color({:?})",
                        i, direction, color
                    );
                }
            }
        }
    }
}

pub struct ParsedConfigState {
    diffuse_color: Vec3,
    specular_color: Vec3,
    shininess: f32,
    vertices: Vec<Vec3>,
}

impl ParsedConfigState {
    pub fn new() -> Self {
        ParsedConfigState {
            diffuse_color: DEFAULT_DIFFUSE_COLOR,
            specular_color: DEFAULT_SPECULAR_COLOR,
            shininess: DEFAULT_SHININESS,
            vertices: Vec::new(),
        }
    }
    pub fn load_config_file(&mut self, file_path: &str) -> Result<Config, String> {
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
            ambient: Vec3::splat(0.0),
            maxdepth: 1,
            maxverts: 0,
            scene_objects: Vec::new(),
            lights: Vec::new(),
        };
        for line in reader.lines() {
            self.parse_line(&line.map_err(|e| e.to_string())?, &mut config)?;
        }
        Ok(config)
    }

    fn parse_line(&mut self, line: &str, config: &mut Config) -> Result<(), String> {
        if line.trim().is_empty() || line.trim_start().starts_with(COMMENT_CHAR) {
            return Ok(());
        }
        let parts: Vec<&str> = line.split(' ').map(|s| s.trim()).collect();
        if parts.len() >= 2 {
            let param = &line[parts[0].len()..].trim();
            match parts[0] {
                "size" => {
                    let (width, height) = self.parse_size(param)?;
                    config.width = width;
                    config.height = height;
                }
                "output" => {
                    let output_file = self.parse_output(param)?;
                    config.output_file = output_file;
                }
                "camera" => {
                    let camera = self.parse_camera(param)?;
                    config.camera = camera;
                }
                "ambient" => {
                    config.ambient = self.parse_ambient(param)?;
                }
                "sphere" => {
                    let sphere = self.parse_sphere(param)?;
                    config.scene_objects.push(sphere);
                }
                "tri" => {
                    let triangle = self.parse_triangle(param)?;
                    config.scene_objects.push(triangle);
                }
                "plane" => {
                    let plane = self.parse_plane(param)?;
                    config.scene_objects.push(plane);
                }
                "point" => {
                    let light = self.parse_point_light(param)?;
                    config.lights.push(light);
                }
                "directional" => {
                    let light = self.parse_directional_light(param)?;
                    config.lights.push(light);
                }
                "diffuse" => {
                    self.diffuse_color = self.parse_simple_vec3(param)?;
                    ParsedConfigState::check_rgb_values(
                        self.diffuse_color.x,
                        self.diffuse_color.y,
                        self.diffuse_color.z,
                    )?;
                    if (self.diffuse_color.x + config.ambient.x) > 1.0
                        || (self.diffuse_color.y + config.ambient.y) > 1.0
                        || (self.diffuse_color.z + config.ambient.z) > 1.0
                    {
                        return Err(
                            "Sum of diffuse color and ambient light components must not exceed 1.0"
                                .to_string(),
                        );
                    }
                }
                "specular" => {
                    self.specular_color = self.parse_simple_vec3(param)?;
                    if self.specular_color.x < 0.0
                        || self.specular_color.y < 0.0
                        || self.specular_color.z < 0.0
                    {
                        return Err("Specular color components must be non-negative".to_string());
                    }
                }
                "shininess" => {
                    self.shininess = param.parse::<f32>().map_err(|e| e.to_string())?;
                    if self.shininess < 0.0 {
                        return Err("Shininess must be non-negative".to_string());
                    }
                }
                "maxdepth" => {
                    config.maxdepth = param.parse::<u32>().map_err(|e| e.to_string())?;
                }
                "maxverts" => {
                    config.maxverts = param.parse::<u32>().map_err(|e| e.to_string())?;
                    self.vertices.reserve(config.maxverts as usize);
                }
                "vertex" => {
                    let vertex = self.parse_simple_vec3(param)?;
                    if self.vertices.len() >= config.maxverts as usize {
                        return Err("Exceeded maximum number of vertices (maxverts)".to_string());
                    }
                    self.vertices.push(vertex);
                }
                _ => {
                    return Err(format!("Unknown configuration key: {}", parts[0]));
                }
            }
        }
        Ok(())
    }
    fn parse_size(&self, value: &str) -> Result<(u32, u32), String> {
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

    fn parse_point_light(&self, value: &str) -> Result<Light, String> {
        let params: Vec<&str> = value.split(' ').collect();
        if params.len() != 6 {
            return Err("Invalid point light format".to_string());
        }
        let position = Vec3::new(
            params[0].parse::<f32>().map_err(|e| e.to_string())?,
            params[1].parse::<f32>().map_err(|e| e.to_string())?,
            params[2].parse::<f32>().map_err(|e| e.to_string())?,
        );
        let color = Vec3::new(
            params[3].parse::<f32>().map_err(|e| e.to_string())?,
            params[4].parse::<f32>().map_err(|e| e.to_string())?,
            params[5].parse::<f32>().map_err(|e| e.to_string())?,
        );

        ParsedConfigState::check_rgb_values(color.x, color.y, color.z)?;

        Ok(Light::Point { position, color })
    }

    fn parse_directional_light(&self, value: &str) -> Result<Light, String> {
        let params: Vec<&str> = value.split(' ').collect();
        if params.len() != 6 {
            return Err("Invalid directional light format".to_string());
        }
        let direction = Vec3::new(
            params[0].parse::<f32>().map_err(|e| e.to_string())?,
            params[1].parse::<f32>().map_err(|e| e.to_string())?,
            params[2].parse::<f32>().map_err(|e| e.to_string())?,
        );
        let color = Vec3::new(
            params[3].parse::<f32>().map_err(|e| e.to_string())?,
            params[4].parse::<f32>().map_err(|e| e.to_string())?,
            params[5].parse::<f32>().map_err(|e| e.to_string())?,
        );

        ParsedConfigState::check_rgb_values(color.x, color.y, color.z)?;

        Ok(Light::Directional {
            direction: direction.normalize(),
            color,
        })
    }

    fn parse_camera(&self, value: &str) -> Result<Camera, String> {
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

    fn parse_ambient(&self, value: &str) -> Result<Vec3, String> {
        let comps: Vec<&str> = value.split(' ').collect();
        if comps.len() != 3 {
            return Err("Invalid ambient light format".to_string());
        }
        let r = comps[0].parse::<f32>().map_err(|e| e.to_string())?;
        let g = comps[1].parse::<f32>().map_err(|e| e.to_string())?;
        let b = comps[2].parse::<f32>().map_err(|e| e.to_string())?;

        ParsedConfigState::check_rgb_values(r, g, b)?;

        Ok(Vec3::new(r, g, b))
    }

    fn parse_simple_vec3(&self, value: &str) -> Result<Vec3, String> {
        let comps: Vec<&str> = value.split(' ').collect();
        if comps.len() != 3 {
            return Err("Invalid Vec3 format".to_string());
        }
        let x: f32 = comps[0].parse::<f32>().map_err(|e| e.to_string())?;
        let y = comps[1].parse::<f32>().map_err(|e| e.to_string())?;
        let z = comps[2].parse::<f32>().map_err(|e| e.to_string())?;
        Ok(Vec3::new(x, y, z))
    }

    fn parse_output(&self, value: &str) -> Result<String, String> {
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

    fn parse_sphere(&self, value: &str) -> Result<Shape, String> {
        // position + radius
        let params: Vec<&str> = value.split(' ').collect();
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
        Ok(Shape::Sphere {
            center,
            radius,
            diffuse_color: self.diffuse_color,
            specular_color: self.specular_color,
            shininess: self.shininess,
            node_index: 0,
        })
    }

    fn parse_triangle(&self, value: &str) -> Result<Shape, String> {
        let params: Vec<&str> = value.split(' ').collect();
        if params.len() != 3 {
            return Err("Invalid triangle format".to_string());
        }
        let v0_index = params[0].parse::<usize>().map_err(|e| e.to_string())?;
        let v1_index = params[1].parse::<usize>().map_err(|e| e.to_string())?;
        let v2_index = params[2].parse::<usize>().map_err(|e| e.to_string())?;

        if v0_index >= self.vertices.len()
            || v1_index >= self.vertices.len()
            || v2_index >= self.vertices.len()
        {
            return Err("Triangle vertex index out of bounds".to_string());
        }

        Ok(Shape::Triangle {
            v0: self.vertices[v0_index],
            v1: self.vertices[v1_index],
            v2: self.vertices[v2_index],
            diffuse_color: self.diffuse_color,
            specular_color: self.specular_color,
            shininess: self.shininess,
            node_index: 0,
        })
    }

    fn parse_plane(&self, value: &str) -> Result<Shape, String> {
        let params: Vec<&str> = value.split(' ').collect();
        if params.len() != 6 {
            return Err("Invalid plane format".to_string());
        }
        let point = Vec3::new(
            params[0].parse::<f32>().map_err(|e| e.to_string())?,
            params[1].parse::<f32>().map_err(|e| e.to_string())?,
            params[2].parse::<f32>().map_err(|e| e.to_string())?,
        );
        let normal = Vec3::new(
            params[3].parse::<f32>().map_err(|e| e.to_string())?,
            params[4].parse::<f32>().map_err(|e| e.to_string())?,
            params[5].parse::<f32>().map_err(|e| e.to_string())?,
        )
        .normalize();

        Ok(Shape::Plane {
            point,
            normal,
            diffuse_color: self.diffuse_color,
            specular_color: self.specular_color,
            shininess: self.shininess,
            node_index: 0,
        })
    }
}
// test

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_size() {
        let parsed_config = ParsedConfigState::new();
        let (width, height) = parsed_config.parse_size("1920 1080").unwrap();
        assert_eq!(width, 1920);
        assert_eq!(height, 1080);
    }

    #[test]
    fn test_parse_camera() {
        let parsed_config = ParsedConfigState::new();
        let camera = parsed_config
            .parse_camera("0.0 0.0 150.0 0.0 0.0 5.0 0.0 1.0 0.0 60")
            .unwrap();
        assert_eq!(camera.position, Vec3::new(0.0, 0.0, 150.0));
        assert_eq!(camera.look_at, Vec3::new(0.0, 0.0, 5.0));
        assert_eq!(camera.up, Vec3::new(0.0, 1.0, 0.0));
        assert_eq!(camera.fov, 60.0);
    }

    #[test]
    fn test_parse_ambient() {
        let parsed_config = ParsedConfigState::new();
        let ambient = parsed_config.parse_ambient("0.2 0.3 0.4").unwrap();
        assert_eq!(ambient, Vec3::new(0.2, 0.3, 0.4));
    }

    #[test]
    fn test_parse_output() {
        let parsed_config = ParsedConfigState::new();
        let output_file = parsed_config.parse_output("final.png").unwrap();
        assert_eq!(output_file, "final.png");
    }
}
