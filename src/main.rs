
use image::GenericImageView;
use std::path::Path;

struct Image {
    width: u32,
    height: u32,
    data: Vec<u32>, 
}

impl Image {
    fn new(width: u32, height: u32, data: Vec<u32>) -> Self {
        Self { width, height, data }
    }

    fn compare(img1 : &Image, img2 : &Image) -> Result<Image, String> {
        if img1.height != img2.height || img1.width != img2.width {
            return Err("Images have differet dimensions".to_string());
        }
        let mut diff_pixels : Vec<u32> = Vec::new();
        for (p1, p2) in img1.data.iter().zip(&img2.data) {
            if *p1 != *p2 {
                let diff = ((*p1 as i32) - (*p2 as i32)).abs() as u32;
                diff_pixels.push(diff);
            } else {
                diff_pixels.push(0);
            }
        }
        let diff_image = Image::new(img1.width, img1.height, diff_pixels);
        return Ok(diff_image);
    }
}

fn file_to_image(path : &str) -> Result<Image, String> {
    let img = image::open(&Path::new(path)).map_err(|e| e.to_string())?;
    let (width, height) = img.dimensions();
    let mut data = Vec::new();
    for y in 0..height {
        for x in 0..width {
            // need to convert pixel to u32
            let pixel = img.get_pixel(x, y);
            let pixel_value = ((pixel[0] as u32) << 16) |
                              ((pixel[1] as u32) << 8)  |
                              (pixel[2] as u32);
            data.push(pixel_value);
        }
    }
    Ok(Image::new(width, height, data))
}

fn save_image(img : &Image, path : &str) -> Result<(), String> {
    let mut imgbuf = image::RgbImage::new(img.width, img.height);
    for y in 0..img.height {
        for x in 0..img.width {
            let pixel_value = img.data[(y * img.width + x) as usize];
            let r = ((pixel_value >> 16) & 0xFF) as u8;
            let g = ((pixel_value >> 8) & 0xFF) as u8;
            let b = (pixel_value & 0xFF) as u8;
            imgbuf.put_pixel(x, y, image::Rgb([r, g, b]));
        }
    }
    imgbuf.save(path).map_err(|e| e.to_string())
}

fn main() {
    let image_path1 = "test_compare_image1.png";
    let image_path2 = "test_compare_image2.png";

    let img1 = match file_to_image(image_path1) {
        Ok(img) => img,
        Err(e) => {
            println!("Error loading image 1: {}", e);
            return;
        }
    };

    let img2 = match file_to_image(image_path2) {
        Ok(img) => img,
        Err(e) => {
            println!("Error loading image 2: {}", e);
            return;
        }
    };

    match Image::compare(&img1, &img2) {
        Err(e) => println!("Error comparing images: {}", e),
        Ok(diff_image) => {
            let output_path = "diff_image.png";
            match save_image(&diff_image, output_path) {
                Ok(_) => println!("Difference image saved to {}", output_path),
                Err(e) => println!("Error saving difference image: {}", e),
            }
        }
    }
}