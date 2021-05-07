use image::{RgbImage, DynamicImage};
use crate::models::mpc::MemeMessage;

pub fn embed_message_into_image(image: &DynamicImage, msg: &MemeMessage) -> Option<RgbImage> {

    let mut converted_image = image.to_rgb8();

    // Unset all bits in the image
    for pixel in converted_image.pixels_mut() {
        let new_r = pixel.0[0] & 0b1111_1110;
        *pixel = image::Rgb([new_r, pixel.0[1], pixel.0[2]]);
    }

    // Get the bytes for the message
    let msg_bytes = serde_json::to_string(&msg).unwrap().into_bytes();
    if msg_bytes.len() > (converted_image.len() / 8) {
        println!("Insufficient image size {} for payload length {}", (converted_image.len() / 8), msg_bytes.len());
        return None;
    }

    let mut image_iterator = converted_image.pixels_mut();

    // Iterate over the bit
    msg_bytes.iter().for_each(|byte| for bit_pos in 0..8 {
        let next_bit = (byte & ( 1 << bit_pos )) > 0;
        let pixel = image_iterator.next().unwrap();

        let new_r = if next_bit {
            pixel.0[0] | 0b0000_0001
        } else {
            pixel.0[0] & 0b1111_1110
        };

        *pixel = image::Rgb([new_r, pixel.0[1], pixel.0[2]]);
    });

    Some(converted_image)
}