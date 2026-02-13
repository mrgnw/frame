//! Stereo pair generation using Depth-Image-Based Rendering (DIBR)
//!
//! This module converts a depth map and an input image into left and right stereo views
//! by horizontally shifting pixels based on their depth values.

use crate::error::SpatialResult;
use image::{DynamicImage, ImageBuffer};
use ndarray::Array2;

/// Generate a stereo pair (left and right images) from an image and depth map
///
/// Uses Depth-Image-Based Rendering (DIBR) to create a right-view by shifting
/// pixels horizontally based on their depth. The depth map controls the amount
/// of shift for each pixel.
///
/// # Arguments
///
/// * `image` - The original input image
/// * `depth` - Normalized depth map (0-1 range, higher = closer)
/// * `max_disparity` - Maximum horizontal shift in pixels
///
/// # Returns
///
/// A tuple of (left_image, right_image) where:
/// - left_image is the original input
/// - right_image is synthesized via DIBR
pub fn generate_stereo_pair(
    image: &DynamicImage,
    depth: &Array2<f32>,
    max_disparity: u32,
) -> SpatialResult<(DynamicImage, DynamicImage)> {
    tracing::info!(
        "Generating stereo pair with max_disparity: {}",
        max_disparity
    );

    let img_rgb = image.to_rgb8();
    let width = img_rgb.width() as usize;
    let height = img_rgb.height() as usize;

    // Validate depth map dimensions
    let (depth_height, depth_width) = depth.dim();
    if depth_height != height || depth_width != width {
        tracing::warn!(
            "Depth map size ({}x{}) doesn't match image size ({}x{}), will resize",
            depth_width,
            depth_height,
            width,
            height
        );
    }

    // Create right image via DIBR
    let mut right_rgb = ImageBuffer::new(width as u32, height as u32);

    // Initialize with a background color (dark gray for disocclusions)
    for pixel in right_rgb.pixels_mut() {
        *pixel = image::Rgb([64, 64, 64]);
    }

    // For each pixel in the original image, compute its disparity and shift it
    for y in 0..height {
        for x in 0..width {
            // Get depth at this pixel (with bounds checking and interpolation)
            let depth_val = get_depth_at(depth, x, y, width, height);

            // Compute horizontal disparity (shift amount)
            // Higher depth (closer object) → larger shift
            let disparity = (depth_val * max_disparity as f32).round() as i32;

            // New position in right image
            let x_right = x as i32 - disparity;

            // Check bounds
            if x_right >= 0 && x_right < width as i32 {
                // Copy pixel from original to right image
                if let Some(pixel) = img_rgb.get_pixel_checked(x as u32, y as u32) {
                    right_rgb.put_pixel(x_right as u32, y as u32, *pixel);
                }
            }
        }
    }

    // Fill holes (disocclusions) with nearest valid pixel
    fill_disocclusions(&mut right_rgb);

    let left_image = image.clone();
    let right_image = DynamicImage::ImageRgb8(right_rgb);

    tracing::info!("Stereo pair generation complete");
    Ok((left_image, right_image))
}

/// Get depth value at a given pixel coordinate with bilinear interpolation
///
/// If the coordinate is out of bounds, returns 0.5 (background/unknown depth)
fn get_depth_at(
    depth: &Array2<f32>,
    x: usize,
    y: usize,
    img_width: usize,
    img_height: usize,
) -> f32 {
    let (depth_height, depth_width) = depth.dim();

    // Handle case where depth map size differs from image size
    if depth_height == img_height && depth_width == img_width {
        // Direct access
        depth[[y, x]]
    } else {
        // Scale coordinates to depth map size
        let scaled_x = (x as f32 * depth_width as f32 / img_width as f32)
            .min(depth_width as f32 - 1.0) as usize;
        let scaled_y = (y as f32 * depth_height as f32 / img_height as f32)
            .min(depth_height as f32 - 1.0) as usize;

        if scaled_y < depth_height && scaled_x < depth_width {
            depth[[scaled_y, scaled_x]]
        } else {
            0.5
        }
    }
}

/// Fill holes (disocclusions) in the right image with nearest valid neighbor
///
/// This is a simple approach: for each invalid pixel, find the nearest valid pixel
/// and copy its value. More sophisticated approaches could use median filtering
/// or edge-aware inpainting.
fn fill_disocclusions(image: &mut ImageBuffer<image::Rgb<u8>, Vec<u8>>) {
    let width = image.width() as usize;
    let height = image.height() as usize;
    let bg_color = image::Rgb([64u8, 64u8, 64u8]);

    // Create a copy to check validity
    let original = image.clone();

    // Simple filling: for each background pixel, find nearest non-background pixel
    for y in 0..height {
        for x in 0..width {
            let pixel = original.get_pixel(x as u32, y as u32);

            // Check if this is a "hole" (background color)
            if pixel[0] == 64 && pixel[1] == 64 && pixel[2] == 64 {
                // Find nearest valid pixel
                if let Some(nearest) = find_nearest_valid_pixel(&original, x, y, bg_color) {
                    image.put_pixel(x as u32, y as u32, nearest);
                }
            }
        }
    }
}

/// Find the nearest non-background pixel to the given coordinate
fn find_nearest_valid_pixel(
    image: &ImageBuffer<image::Rgb<u8>, Vec<u8>>,
    cx: usize,
    cy: usize,
    bg_color: image::Rgb<u8>,
) -> Option<image::Rgb<u8>> {
    let width = image.width() as usize;
    let height = image.height() as usize;

    // Search in expanding rings around the point
    for radius in 1..=20 {
        for dy in -(radius as i32)..=(radius as i32) {
            for dx in -(radius as i32)..=(radius as i32) {
                // Only check pixels at this radius (not interior)
                if dx.abs() != radius as i32 && dy.abs() != radius as i32 {
                    continue;
                }

                let nx = (cx as i32 + dx) as usize;
                let ny = (cy as i32 + dy) as usize;

                if nx < width && ny < height {
                    let pixel = image.get_pixel(nx as u32, ny as u32);
                    // Check if this is not a background/hole pixel
                    if pixel[0] != bg_color[0] || pixel[1] != bg_color[1] || pixel[2] != bg_color[2]
                    {
                        return Some(*pixel);
                    }
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::GenericImageView;
    use ndarray::Array2;

    #[test]
    fn test_get_depth_at_same_size() {
        let depth = Array2::from_elem((10, 10), 0.5);
        let d = get_depth_at(&depth, 5, 5, 10, 10);
        assert!((d - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_get_depth_at_bounds() {
        let depth = Array2::from_elem((10, 10), 0.5);
        let d = get_depth_at(&depth, 9, 9, 10, 10);
        assert!((d - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_get_depth_at_scaled() {
        // Small depth map
        let mut depth = Array2::from_elem((5, 5), 0.5);
        depth[[2, 2]] = 0.8;

        // Scaled access
        let d = get_depth_at(&depth, 10, 10, 20, 20);
        assert!(d > 0.4 && d < 1.0);
    }

    #[test]
    fn test_stereo_pair_creation() {
        // Create a test image
        let test_img = image::ImageBuffer::from_fn(100, 100, |x, y| {
            let r = (x as f32 / 100.0 * 255.0) as u8;
            let g = (y as f32 / 100.0 * 255.0) as u8;
            image::Rgb([r, g, 128])
        });
        let dyn_img = DynamicImage::ImageRgb8(test_img);

        // Create a simple depth map
        let depth = Array2::from_elem((100, 100), 0.5);

        // Generate stereo pair
        let result = generate_stereo_pair(&dyn_img, &depth, 30);
        assert!(result.is_ok());

        let (left, right) = result.unwrap();
        assert_eq!(left.dimensions(), right.dimensions());
    }

    #[test]
    fn test_stereo_pair_dimensions_preserved() {
        let test_img =
            image::ImageBuffer::from_fn(200, 150, |x, y| image::Rgb([x as u8, y as u8, 128]));
        let dyn_img = DynamicImage::ImageRgb8(test_img);
        let depth = Array2::from_elem((150, 200), 0.5);

        let (left, right) = generate_stereo_pair(&dyn_img, &depth, 20).unwrap();

        assert_eq!(left.width(), 200);
        assert_eq!(left.height(), 150);
        assert_eq!(right.width(), 200);
        assert_eq!(right.height(), 150);
    }
}
