mod config_builder;
mod camera;
mod shape;
mod shapes;
mod light;
pub use config_builder::{load_config_file, Config};
pub use shape::{Shape, Ray};