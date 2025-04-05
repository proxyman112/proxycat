use image::{Rgba, RgbaImage};
use std::fs::File;
use std::io::BufWriter;
use log::{info, debug};
use crate::error::{Result, ProxyCatError};

pub fn create_icon() -> Result<()> {
    info!("Creating application icon...");
    let size = 32u32;
    let mut img = RgbaImage::new(size, size);
    
    // Create a simple cat icon - draw a filled circle
    for y in 0..size {
        for x in 0..size {
            let center_x = size as f32 / 2.0;
            let center_y = size as f32 / 2.0;
            let distance = ((x as f32 - center_x).powi(2) + 
                          (y as f32 - center_y).powi(2)).sqrt();
            
            if distance <= size as f32 / 2.5 {
                // Main circle (gray-blue)
                img.put_pixel(x, y, Rgba([120, 140, 180, 255]));
            } else if distance <= size as f32 / 2.0 {
                // Border (darker blue)
                img.put_pixel(x, y, Rgba([0, 90, 200, 255]));
            } else {
                // Transparent background
                img.put_pixel(x, y, Rgba([0, 0, 0, 0]));
            }
        }
    }

    // Add cat ears
    for y in 0..size/3 {
        for x in 0..size {
            let left_ear = ((x as f32 - size as f32 / 4.0).powi(2) + 
                          (y as f32 - size as f32 / 4.0).powi(2)).sqrt();
            let right_ear = ((x as f32 - 3.0 * size as f32 / 4.0).powi(2) + 
                           (y as f32 - size as f32 / 4.0).powi(2)).sqrt();
            
            if left_ear <= size as f32 / 6.0 || right_ear <= size as f32 / 6.0 {
                img.put_pixel(x, y, Rgba([0, 120, 255, 255]));
            }
        }
    }
    // Add cat eyes
    for y in size/3..2*size/3 {
        for x in 0..size {
            let left_eye = ((x as f32 - size as f32 / 3.0).powi(2) + 
                          (y as f32 - size as f32 / 2.0).powi(2)).sqrt();
            let right_eye = ((x as f32 - 2.0 * size as f32 / 3.0).powi(2) + 
                           (y as f32 - size as f32 / 2.0).powi(2)).sqrt();
            
            if left_eye <= size as f32 / 8.0 || right_eye <= size as f32 / 8.0 {
                img.put_pixel(x, y, Rgba([0, 0, 0, 255])); // Black eyes
            }
        }
    }
    // Add cat nose
    for y in size/2..2*size/3 {
        for x in size/3..2*size/3 {
            let nose = ((x as f32 - size as f32 / 2.0).powi(2) + 
                       (y as f32 - 3.0 * size as f32 / 5.0).powi(2)).sqrt();
            
            if nose <= size as f32 / 10.0 {
                img.put_pixel(x, y, Rgba([255, 150, 150, 255])); // Pink nose
            }
        }
    }

    // Add cat whiskers
    for i in 0..3 {
        let y_offset = size as f32 / 2.0 + (i as f32 - 1.0) * 3.0;
        
        // Left whiskers
        for x in (size/6..size/2).step_by(1) {
            let y = y_offset as u32;
            if x < size && y < size {
                img.put_pixel(x, y, Rgba([0, 90, 200, 255]));
            }
        }

        // Right whiskers 
        for x in (size/2..5*size/6).step_by(1) {
            let y = y_offset as u32;
            if x < size && y < size {
                img.put_pixel(x, y, Rgba([0, 90, 200, 255]));
            }
        }
    }

    // Save as ICO file
    debug!("Saving icon to file...");
    let file = File::create("icon.ico")
        .map_err(|e| ProxyCatError::Icon(format!("Failed to create icon file: {}", e)))?;
    let writer = BufWriter::new(file);
    img.write_with_encoder(image::codecs::ico::IcoEncoder::new(writer))
        .map_err(|e| ProxyCatError::Icon(format!("Failed to write icon: {}", e)))?;
    
    info!("Icon created successfully");
    Ok(())
} 