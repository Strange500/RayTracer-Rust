//! Image comparison module for ray tracer
//!
//! This module provides utilities for loading, comparing, and saving images.
//! Images are represented in RGB format with 8 bits per channel, packed into u32.

use image::GenericImageView;
use std::path::Path;

// Bit shift and mask constants for RGB channel extraction
const RED_SHIFT: u32 = 16;
const GREEN_SHIFT: u32 = 8;
const CHANNEL_MASK: u32 = 0xFF;

/// Represents an RGB image with packed pixel data
///
/// Each pixel is stored as a u32 in the format 0x00RRGGBB where:
/// - RR: Red channel (8 bits)
/// - GG: Green channel (8 bits)
/// - BB: Blue channel (8 bits)
#[derive(Debug, PartialEq)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u32>,
}

impl Image {
    /// Creates a new Image with the specified dimensions and pixel data
    ///
    /// # Arguments
    /// * `width` - Width of the image in pixels
    /// * `height` - Height of the image in pixels
    /// * `data` - Vector of packed RGB pixels (length must equal width * height)
    pub fn new(width: u32, height: u32, data: Vec<u32>) -> Self {
        Self {
            width,
            height,
            data,
        }
    }

    /// Compares two images and returns a difference image
    ///
    /// For each pixel, calculates the absolute difference for each RGB channel.
    /// If images have different dimensions, returns an error.
    ///
    /// # Arguments
    /// * `img1` - First image to compare
    /// * `img2` - Second image to compare
    ///
    /// # Returns
    /// * `Ok(Image)` - Difference image where each channel contains the absolute difference
    /// * `Err(String)` - Error message if dimensions don't match
    pub fn compare(img1: &Image, img2: &Image) -> Result<(u128, Image), String> {
        if img1.height != img2.height || img1.width != img2.width {
            return Err("Images have different dimensions".to_string());
        }

        let mut diff_pixels: Vec<u32> = Vec::with_capacity(img1.data.len());
        let mut total_diff: u128 = 0;

        for (p1, p2) in img1.data.iter().zip(&img2.data) {
            let diff = if *p1 != *p2 {
                // Extract RGB channels from each pixel
                let (r1, g1, b1) = extract_rgb(*p1);
                let (r2, g2, b2) = extract_rgb(*p2);

                // Calculate absolute difference for each channel
                let r_diff = (r1 as i32 - r2 as i32).unsigned_abs();
                let g_diff = (g1 as i32 - g2 as i32).unsigned_abs();
                let b_diff = (b1 as i32 - b2 as i32).unsigned_abs();
                println!("Diff R:{} G:{} B:{}", r_diff, g_diff, b_diff);
                // return 0 if diff is < 1 per channel
                if r_diff <= 1 && g_diff <= 1 && b_diff <= 1 {
                    0
                } else {
                    pack_rgb(r_diff, g_diff, b_diff)
                }
            } else {
                0
            };
            if diff != 0 {
                total_diff += 1;
            }
            diff_pixels.push(diff);
        }

        Ok((total_diff, Image::new(img1.width, img1.height, diff_pixels)))
    }
}

/// Extracts RGB channels from a packed pixel value
///
/// # Arguments
/// * `pixel` - Packed RGB pixel in 0x00RRGGBB format
///
/// # Returns
/// Tuple of (red, green, blue) channel values
#[inline]
pub fn extract_rgb(pixel: u32) -> (u32, u32, u32) {
    let r = (pixel >> RED_SHIFT) & CHANNEL_MASK;
    let g = (pixel >> GREEN_SHIFT) & CHANNEL_MASK;
    let b = pixel & CHANNEL_MASK;
    (r, g, b)
}

/// Packs RGB channel values into a single pixel value
///
/// # Arguments
/// * `r` - Red channel value (0-255)
/// * `g` - Green channel value (0-255)
/// * `b` - Blue channel value (0-255)
///
/// # Returns
/// Packed RGB pixel in 0x00RRGGBB format
#[inline]
fn pack_rgb(r: u32, g: u32, b: u32) -> u32 {
    (r << RED_SHIFT) | (g << GREEN_SHIFT) | b
}

/// Loads an image from a file
///
/// # Arguments
/// * `path` - Path to the image file
///
/// # Returns
/// * `Ok(Image)` - Successfully loaded image
/// * `Err(String)` - Error message if loading fails
pub fn file_to_image(path: &str) -> Result<Image, String> {
    let img = image::open(Path::new(path)).map_err(|e| e.to_string())?;
    let (width, height) = img.dimensions();
    let mut data = Vec::with_capacity((width * height) as usize);

    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel(x, y);
            let pixel_value = pack_rgb(pixel[0] as u32, pixel[1] as u32, pixel[2] as u32);
            data.push(pixel_value);
        }
    }
    Ok(Image::new(width, height, data))
}

/// Saves an image to a file
///
/// # Arguments
/// * `img` - Image to save
/// * `path` - Destination file path
///
/// # Returns
/// * `Ok(())` - Image saved successfully
/// * `Err(String)` - Error message if saving fails
pub fn save_image(img: &Image, path: &str) -> Result<(), String> {
    let mut imgbuf = image::RgbImage::new(img.width, img.height);

    for y in 0..img.height {
        for x in 0..img.width {
            let pixel_value = img.data[(y * img.width + x) as usize];
            let (r, g, b) = extract_rgb(pixel_value);
            imgbuf.put_pixel(x, y, image::Rgb([r as u8, g as u8, b as u8]));
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
        let (_diff, diff_img) = result.unwrap();
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
        assert_eq!(
            result.err(),
            Some("Images have different dimensions".to_string())
        );
    }

    #[test]
    fn test_compare_calculates_difference_correctly() {
        // Arrange
        // Pixel 1: Pure Red (0xFF0000) vs Black (0x000000) -> Diff should be Red
        // Pixel 2: Pure White (0xFFFFFF) vs Pure Blue (0x0000FF) -> Diff should be RG (0xFFFF00)
        let img1 = Image::new(2, 1, vec![0xFF0000, 0xFFFFFF]);
        let img2 = Image::new(2, 1, vec![0x000000, 0x0000FF]);

        // Act
        let (_diff, img) = Image::compare(&img1, &img2).unwrap();

        // Assert
        // 1. FF0000 - 000000 = FF0000
        // 2. FFFFFF - 0000FF = FFFF00 (Red diff=FF, Green diff=FF, Blue diff=0)
        assert_eq!(img.data[0], 0xFF0000);
        assert_eq!(img.data[1], 0xFFFF00);
    }

    #[test]
    fn test_compare_channel_borrowing() {
        // This test proves why we need channel-wise math.
        // If we just did (p1 - p2), 0x010000 - 0x00FFFF would be 1 (Blue).
        // Correctly, it should be: Red diff=1, Green diff=255, Blue diff=255.

        let img1 = Image::new(1, 1, vec![0x010000]); // Red = 1
        let img2 = Image::new(1, 1, vec![0x00FFFF]); // Green=255, Blue=255

        let (_diff, img) = Image::compare(&img1, &img2).unwrap();

        // Expected:
        // R: |1 - 0| = 1
        // G: |0 - 255| = 255 (FF)
        // B: |0 - 255| = 255 (FF)
        // Result: 0x01FFFF
        assert_eq!(img.data[0], 0x01FFFF);
    }
}
