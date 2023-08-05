#[cfg(test)]
use std::fs;

#[cfg(test)]
use image::{Rgb, RgbImage};

#[cfg(test)]
pub const BLACK: Rgb<u8> = Rgb([0, 0, 0]);
#[cfg(test)]
pub const RED: Rgb<u8> = Rgb([255, 0, 0]);
#[cfg(test)]
pub const BLUE: Rgb<u8> = Rgb([0, 0, 255]);
#[cfg(test)]
pub const GREEN: Rgb<u8> = Rgb([0, 255, 0]);
#[cfg(test)]
pub const PURPLE: Rgb<u8> = Rgb([255, 64, 255]);
#[cfg(test)]
const TEST_RESULT_DIR: &str = "./mosaic_tests/";

#[cfg(test)]
pub fn create_with_colour(width: u32, height: u32, colour: Rgb<u8>) -> RgbImage {
    let mut img = RgbImage::new(width, height);

    for x in 0..width {
        for y in 0..height {
            img.put_pixel(x, y, colour);
            img.put_pixel(x, y, colour);
        }
    }

    img
}

#[cfg(test)]
pub fn is_colour_at_pixel(x: u32, y: u32, image: &RgbImage, colour: Rgb<u8>) -> bool {
    image.get_pixel(x, y).eq(&colour)
}

#[cfg(test)]
pub fn is_colour_in_range(start_x: u32, start_y: u32, end_x: u32, end_y: u32, image: &RgbImage, colour: Rgb<u8>) -> bool {
    for x in start_x..end_x {
        for y in start_y..end_y {
            if !is_colour_at_pixel(x, y, image, colour) {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
pub fn has_black_vertical_line(x: u32, image: &RgbImage) -> bool {
    is_colour_in_range(x, 0, x, image.height(), image, BLACK)
}

#[cfg(test)]
pub fn has_black_horizontal_line(y: u32, image: &RgbImage) -> bool {
    is_colour_in_range(0, y, image.width(), y, image, BLACK)
}

#[cfg(test)]
pub fn has_black_vertical_line_partial(x: u32, start_y: u32, end_y: u32, image: &RgbImage) -> bool {
    is_colour_in_range(x, start_y, x, end_y, image, BLACK)
}

#[cfg(test)]
pub fn has_black_horizontal_line_partial(y: u32, start_x: u32, end_x: u32, image: &RgbImage) -> bool {
    is_colour_in_range(start_x, y, end_x, y, image, BLACK)
}

#[cfg(test)]
pub fn save_result(result: &RgbImage, filename: &str) {
    let file_path = [TEST_RESULT_DIR, filename, ".png"].join("");
    fs::create_dir_all(TEST_RESULT_DIR).unwrap();
    result.save(file_path).unwrap();
}
