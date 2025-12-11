// Image and all fucntion are in imgcomparator/comparator.rs
mod config;
mod imgcomparator;
use config::parser::parse_config;
use imgcomparator::{Image, file_to_image, save_image};
fn main() {
    let config_path = "./test.scene";
    let config = parse_config(config_path).expect("Failed to parse config");
}
