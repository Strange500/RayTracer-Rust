use image::GenericImageView;
use std::path::Path;

// Added Debug and PartialEq to allow assert_eq!(img1, img2) in tests
#[derive(Debug, PartialEq)] 
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u32>, 
}

impl Image {
    // Made public so tests/users can create images manually
    pub fn new(width: u32, height: u32, data: Vec<u32>) -> Self {
        Self { width, height, data }
    }

    pub fn compare(img1 : &Image, img2 : &Image) -> Result<Image, String> {
        if img1.height != img2.height || img1.width != img2.width {
            return Err("Images have different dimensions".to_string());
        }
        
        let mut diff_pixels : Vec<u32> = Vec::with_capacity(img1.data.len());
        
        for (p1, p2) in img1.data.iter().zip(&img2.data) {
            if *p1 != *p2 {
                // FIXED: Calculate difference for each channel separately
                // to avoid "borrowing" errors across bytes.
                let r1 = (p1 >> 16) & 0xFF;
                let g1 = (p1 >> 8) & 0xFF;
                let b1 = p1 & 0xFF;

                let r2 = (p2 >> 16) & 0xFF;
                let g2 = (p2 >> 8) & 0xFF;
                let b2 = p2 & 0xFF;

                let r_diff = (r1 as i32 - r2 as i32).abs() as u32;
                let g_diff = (g1 as i32 - g2 as i32).abs() as u32;
                let b_diff = (b1 as i32 - b2 as i32).abs() as u32;

                // Re-pack into u32
                let diff = (r_diff << 16) | (g_diff << 8) | b_diff;
                diff_pixels.push(diff);
            } else {
                diff_pixels.push(0);
            }
        }
        
        Ok(Image::new(img1.width, img1.height, diff_pixels))
    }
}

pub fn file_to_image(path : &str) -> Result<Image, String> {
    // Note: Usually hard to test without mocking file system, 
    // so we rely on integration tests or manual checks for this.
    let img = image::open(&Path::new(path)).map_err(|e| e.to_string())?;
    let (width, height) = img.dimensions();
    let mut data = Vec::new();
    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel(x, y);
            let pixel_value = ((pixel[0] as u32) << 16) |
                              ((pixel[1] as u32) << 8)  |
                              (pixel[2] as u32);
            data.push(pixel_value);
        }
    }
    Ok(Image::new(width, height, data))
}

pub fn save_image(img : &Image, path : &str) -> Result<(), String> {
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

// ==========================================================
// TESTS
// ==========================================================
#[cfg(test)]
mod tests {
    use super::*; // Import everything from the parent module

    #[test]
    fn test_compare_identical_images() {
        // Arrange: Create two identical images
        let data = vec![0xFF0000, 0x00FF00, 0x0000FF]; // Red, Green, Blue
        let img1 = Image::new(3, 1, data.clone());
        let img2 = Image::new(3, 1, data.clone());

        // Act
        let result = Image::compare(&img1, &img2);

        // Assert: Should be Ok, and data should be all 0s
        assert!(result.is_ok());
        let diff_img = result.unwrap();
        assert_eq!(diff_img.data, vec![0, 0, 0]);
        assert_eq!(diff_img.width, 3);
    }

    #[test]
    fn test_compare_dimension_mismatch() {
        // Arrange: Different sizes
        let img1 = Image::new(2, 2, vec![0; 4]);
        let img2 = Image::new(3, 3, vec![0; 9]);

        // Act
        let result = Image::compare(&img1, &img2);

        // Assert: Should be an Error
        assert!(result.is_err());
        assert_eq!(result.err(), Some("Images have different dimensions".to_string()));
    }

    #[test]
    fn test_compare_calculates_difference_correctly() {
        // Arrange
        // Pixel 1: Pure Red (0xFF0000) vs Black (0x000000) -> Diff should be Red
        // Pixel 2: Pure White (0xFFFFFF) vs Pure Blue (0x0000FF) -> Diff should be RG (0xFFFF00)
        let img1 = Image::new(2, 1, vec![0xFF0000, 0xFFFFFF]);
        let img2 = Image::new(2, 1, vec![0x000000, 0x0000FF]);

        // Act
        let result = Image::compare(&img1, &img2).unwrap();

        // Assert
        // 1. FF0000 - 000000 = FF0000
        // 2. FFFFFF - 0000FF = FFFF00 (Red diff=FF, Green diff=FF, Blue diff=0)
        assert_eq!(result.data[0], 0xFF0000); 
        assert_eq!(result.data[1], 0xFFFF00);
    }
    
    #[test]
    fn test_compare_channel_borrowing() {
        // This test proves why we need channel-wise math.
        // If we just did (p1 - p2), 0x010000 - 0x00FFFF would be 1 (Blue).
        // Correctly, it should be: Red diff=1, Green diff=255, Blue diff=255.
        
        let img1 = Image::new(1, 1, vec![0x010000]); // Red = 1
        let img2 = Image::new(1, 1, vec![0x00FFFF]); // Green=255, Blue=255

        let result = Image::compare(&img1, &img2).unwrap();

        // Expected: 
        // R: |1 - 0| = 1
        // G: |0 - 255| = 255 (FF)
        // B: |0 - 255| = 255 (FF)
        // Result: 0x01FFFF
        assert_eq!(result.data[0], 0x01FFFF);
    }
}