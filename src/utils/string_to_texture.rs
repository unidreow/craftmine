use image::{DynamicImage, Rgba, RgbaImage};
use rusttype::{Font, Scale, point};

pub fn create_text_texture(
    text: &str,
    font_data: &[u8],
    font_size: f32,
    text_color: Rgba<u8>,
    background_color: Option<Rgba<u8>>,
) -> Result<DynamicImage, Box<dyn std::error::Error>> {
    // Load the font
    let font = Font::try_from_bytes(font_data).ok_or("Failed to load font")?;
    
    // Set the scale (font size)
    let scale = Scale::uniform(font_size);
    
    // Calculate text dimensions (with some padding)
    let v_metrics = font.v_metrics(scale);
    let glyphs: Vec<_> = font.layout(text, scale, point(0.0, v_metrics.ascent)).collect();
    let width = glyphs.iter().rev().map(|g| g.position().x + g.unpositioned().h_metrics().advance_width).next().unwrap_or(0.0).ceil() as u32;
    let height = (v_metrics.ascent - v_metrics.descent + v_metrics.line_gap).ceil() as u32;
    
    // Create a new RGBA image with transparent background
    let mut image = RgbaImage::new(width, height);
    
    // Fill with background color if provided
    if let Some(bg_color) = background_color {
        for pixel in image.pixels_mut() {
            *pixel = bg_color;
        }
    }
    
    // Draw the text
    for glyph in glyphs {
        if let Some(bounding_box) = glyph.pixel_bounding_box() {
            glyph.draw(|x, y, v| {
                let x = x as i32 + bounding_box.min.x;
                let y = y as i32 + bounding_box.min.y;
                
                // Only draw pixels that are within the image bounds
                if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
                    let pixel = image.get_pixel_mut(x as u32, y as u32);
                    
                    if let Some(bg_color) = background_color {
                        // Blend with background color
                        let alpha = (v * 255.0) as u8;
                        *pixel = Rgba([
                            (text_color[0] as f32 * v + bg_color[0] as f32 * (1.0 - v)) as u8,
                            (text_color[1] as f32 * v + bg_color[1] as f32 * (1.0 - v)) as u8,
                            (text_color[2] as f32 * v + bg_color[2] as f32 * (1.0 - v)) as u8,
                            (text_color[3] as f32 * v + bg_color[3] as f32 * (1.0 - v)) as u8,
                        ]);
                    } else {
                        // No background - apply text color with proper alpha
                        let current_alpha = pixel[3] as f32 / 255.0;
                        let new_alpha = v + current_alpha * (1.0 - v);
                        *pixel = Rgba([
                            (text_color[0] as f32 * v + pixel[0] as f32 * (1.0 - v)) as u8,
                            (text_color[1] as f32 * v + pixel[1] as f32 * (1.0 - v)) as u8,
                            (text_color[2] as f32 * v + pixel[2] as f32 * (1.0 - v)) as u8,
                            (new_alpha * 255.0) as u8,
                        ]);
                    }
                }
            });
        }
    }
    
    Ok(DynamicImage::ImageRgba8(image))
}