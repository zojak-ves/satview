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

    let file = File::open("src/tle2.txt").unwrap();
    let reader = BufReader::new(file); 
    let mut tle_string = String::from("");

    for line in reader.lines() { 
        tle_string.push_str(line.unwrap().as_str());
        tle_string.push_str("\n");
    }

    let mut gateway_phoenix: observer::Observer = observer::Observer::new([33.4484, -112.0740, 0.00]);
    let mut terminal: observer::Observer = observer::Observer::new([48.0, 0.0, 0.00]);

    let gateway_color: [u8; 4] = [255,0,0,255]; // Red 0% transparent.
    let terminal_color: [u8; 4] = [0,255,0,255]; // Green 0% transparent.

    let map_data = init_map();
    let mut out = map_data.0;
    let height = map_data.1;
    let width = map_data.2;

    // Plot Observer point
    let (mut x, mut y) = gimme_xy(&terminal.geodetic_coords.longitude, &terminal.geodetic_coords.latitude, &height, &width);
    let pix_vec = get_pixel_vector(&width, &height, &(x,y), false);
        
    for pixel in &pix_vec { 
        color_pixel(&mut out, pixel, &terminal_color);
    }

    let mut sat_vec: Vec<Satellite> = Vec::new();
    let sat_elements = parse_3les(&tle_string).unwrap();

    for element in sat_elements { 
        let sat_state: satellite::Satellite = Satellite::new(element);
        sat_vec.push(sat_state);
    }   

    for n in 0..=3600 * 24 * 7{
        for satellite in 0..=sat_vec.len()-1 {
            let sat_constants = sgp4::Constants::from_elements(&sat_vec[satellite].sat_elements).unwrap();
            let elapsed_time = n;
            let time_delta = chrono::TimeDelta::seconds(elapsed_time);
            // let time_delta = chrono::TimeDelta::minutes(elapsed_time);
            let new_epoch = sat_vec[satellite].sat_elements.datetime.checked_add_signed(time_delta).unwrap();

            // The propagate function returns position as TEME reference frame coordinates in km and
            // returns velocity in each dimension in terms of km/s
            let prediction: sgp4::Prediction = sat_constants.propagate(sgp4::MinutesSinceEpoch((elapsed_time as f64 / 60.0) as f64))?;
            // Sets satellite coordinates for all reference frames
            sat_vec[satellite].update_sat_state(&prediction, &new_epoch);
            
            // Update observer state to pull in new teme coords with respect to new epoch
            gateway_phoenix.update_state(&new_epoch);
            terminal.update_state(&new_epoch);

            // Set the look angle values
            gateway_phoenix.calculate_look_angle(&prediction, &new_epoch);
            terminal.calculate_look_angle(&prediction, &new_epoch);
            
            // Get sat path coordinates in x,y
            let (mut x, mut y) = gimme_xy(&sat_vec[satellite].geodetic_coordinates.longitude, &sat_vec[satellite].geodetic_coordinates.latitude, &height, &width);
            let pix_vec = get_pixel_vector(&width, &height, &(x,y), true);
            
            // Color the pixels for the sat path.
            for pixel in &pix_vec { 
                color_pixel(&mut out, pixel, &gateway_color);
            }

            if radians_to_degrees(&terminal.look_angle.elevation) >= 15.0 {

                let (mut x, mut y) = gimme_xy(&sat_vec[satellite].geodetic_coordinates.longitude, &sat_vec[satellite].geodetic_coordinates.latitude, &height, &width);
                let pix_vec = get_pixel_vector(&width, &height, &(x,y), true);
            
                for pixel in &pix_vec { 
                    color_pixel(&mut out, pixel, &terminal_color);
                }
            }
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

pub fn color_pixel(image: &mut image::RgbaImage, pixel_coordinates: &(u32, u32), color: &[u8; 4]) { 
    let (width, height) = image.dimensions();

    if pixel_coordinates.0 >= width { 
        return;
    }
    if pixel_coordinates.1 >= height { 
        return;
    }

    image.put_pixel(pixel_coordinates.0, pixel_coordinates.1, image::Rgba(*color));
}

pub fn get_pixel_vector(width: &u32, height: &u32, center_pixel: &(u32, u32), is_sat: bool) -> Vec<(u32, u32)> {
        let mut pixel_vector = vec![];
        let x = center_pixel.0;
        let y = center_pixel.1;
    
        let center      = (x,y);
        pixel_vector.push(center);

        // Only plot a fat pixel for observer locations
        if is_sat == false {
            let top         = if y < *height                {(x, y+1)}   else {(x,y)};
            let bot         = if y > 0                      {(x, y-1)}   else {(x,y)};
            let left        = if x > 0                      {(x-1, y)}   else {(x,y)};
            let right       = if x < *width                 {(x+1, y)}   else {(x,y)};
            let top_right   = if x < *width && y > 0        {(x+1, y-1)} else {(x,y)};
            let top_left    = if x > 0 && y > 0             {(x-1, y-1)} else {(x,y)};
            let bot_right   = if x < *width && y < *height  {(x+1, y+1)} else {(x,y)};
            let bot_left    = if x > 0 && y < *height       {(x-1, y+1)} else {(x,y)};

            pixel_vector.push(top); 
            pixel_vector.push(left);
            pixel_vector.push(right); 
            pixel_vector.push(top_right);
            pixel_vector.push(top_left);
            pixel_vector.push(bot_right);
            pixel_vector.push(bot_left);
        }
        pixel_vector
}

pub fn gimme_xy(lat: &f64, lon: &f64, height: &u32, width: &u32) -> (u32, u32) { 
    let h = *height as f64;
    let w = *width as f64;

    let x = (w as f64 * (180. + lat) / 360.).floor() as u32 ; //row
    let y = (h as f64 * (90. - lon) / 180.).floor() as u32 ; //col

    (x,y)
}

fn init_map() -> (image::RgbaImage, u32, u32) {
    let mut input_image = image::open("BigEarth.jpg").unwrap();
    let (width, height) = input_image.dimensions();
    let mut out: image::RgbaImage = image::ImageBuffer::new(width, height);
    
    // Create the output map.
    for x_iter in 0..input_image.width() { 
        for y_iter in 0..input_image.height() {
            let color = input_image.get_pixel(x_iter,y_iter);
            let pix_tuple = (x_iter, y_iter);
            
            out.put_pixel(x_iter,y_iter,color);
        }
    }
    (out, height, width)
}