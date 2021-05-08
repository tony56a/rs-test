use crate::models::mpc::MemeMessage;
use image::{DynamicImage, RgbImage};

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
        println!(
            "Insufficient image size {} for payload length {}",
            (converted_image.len() / 8),
            msg_bytes.len()
        );
        return None;
    }

    let mut image_iterator = converted_image.pixels_mut();

    // Iterate over the bit
    msg_bytes.iter().for_each(|byte| {
        for bit_pos in 0..8 {
            let next_bit = (byte & (1 << bit_pos)) > 0;
            let pixel = image_iterator.next().unwrap();

            let new_r = if next_bit {
                pixel.0[0] | 0b0000_0001
            } else {
                pixel.0[0] & 0b1111_1110
            };

            *pixel = image::Rgb([new_r, pixel.0[1], pixel.0[2]]);
        }
    });

    Some(converted_image)
}

pub fn decode_message_from_image(image: &DynamicImage) -> Option<MemeMessage> {
    let converted_image = image.to_rgb8();
    let mut image_iterator = converted_image.pixels();

    let mut payload: Vec<u8> = Vec::new();
    let mut pixel_counter = 0;
    'main: loop {
        let mut byte_to_write: u8 = 0;
        for bit_pos in 0..8 {
            let pixel = image_iterator.next().unwrap();
            let bit_set = pixel.0[0] & 0b0000_0001;
            byte_to_write |= bit_set << bit_pos;
            pixel_counter += 1;
            if pixel_counter >= converted_image.len() {
                return None;
            }
        }
        if byte_to_write == 0 {
            break 'main;
        }
        payload.push(byte_to_write);
    }

    let payload_string = String::from_utf8_lossy(payload.as_slice()).to_string();
    match serde_json::from_str::<Option<MemeMessage>>(payload_string.as_str()) {
        Ok(message) => message,
        Err(_) => None,
    }
}
