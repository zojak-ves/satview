use std::fs::File;
use std::io::{BufRead, BufReader};
use image::ImageReader;

use satutil::radians_to_degrees;
use sgp4::parse_3les;
mod satutil;
pub mod observer;
pub mod satellite;
use crate::satellite::Satellite;
pub mod coord_systems;

use image::{GenericImageView, ImageBuffer, RgbaImage, imageops, Rgba};

fn main() -> anyhow::Result<()>{
    /* All of the image processing below will need to go in its own thread. 
       The satellite propagation will also go in its own thread and update 
       the image processing with the lat/long coordinates of the satllite 
       for as long as it is visible to the gateway AND the terminal modems.
       Perhaps add 2 operation types: A. Vis to terminal and gateway. 
       B. vis to just a single observer.

       Also should work on converting all of the sgp4-util code to a library to avoid
       so much duplication. Not important at this time.
    */

    let mut input_image = image::open("BigEarth.jpg").unwrap();
    let (width, height) = input_image.dimensions();
    let mut out: image::RgbaImage = image::ImageBuffer::new(width, height);

    let lat = 0;
    let lon = 0;

    let x = width * (180 + lat) / 360; //row
    let y = height * (90 - lon) / 180; //col

    let center      = (x,y);
    let top         = (x, y+1);
    let bot         = (x, y-1);
    let left        = (x-1, y);
    let right       = (x+1, y);
    let top_right   = (x+1, y-1);
    let top_left    = (x-1, y-1);
    let bot_right   = (x+1, y+1);
    let bot_left    = (x-1, y+1);

    let pix_vec = vec![center, top, bot, left, right, top_right, top_left, bot_right, bot_left];

    for x_iter in 0..input_image.width() { 
        for y_iter in 0..input_image.height() {
            let color = input_image.get_pixel(x_iter,y_iter);
            let pix_tuple = (x_iter, y_iter);
            let mut matched_val: bool = false;

            if is_matched(&pix_vec, &pix_tuple) {
                out.put_pixel(x_iter,y_iter,image::Rgba([255,0,0,255]));

                continue;
            }

            out.put_pixel(x_iter,y_iter,color);
        }
    }

    out.save("out.png").unwrap();
    
    Ok(())
}

pub fn is_matched(pix_vec: &Vec<(u32,u32)>, pix_tuple: &(u32, u32)) -> bool { 
    for iter in pix_vec { 
        if  pix_tuple.0 == iter.0 && 
            pix_tuple.1 == iter.1 { 
            println!("Match {:?}, {:?}", pix_tuple, iter);
            return true;
        }
    }
    false
}