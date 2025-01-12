use std::fs::File; 
use sgp4::parse_3les;
use std::io::{BufRead, BufReader};
use image::GenericImageView;
use std::thread;

mod utils; pub use utils::coordinate_systems;
mod obs; use obs::observer::Observer;
mod sat; use sat::satellite::Satellite;

fn main() -> anyhow::Result<()> {
    
    let earth_file = "BigEarth.jpg";
    let default_img = image::ImageBuffer::new(0,0);
    let mut img : image::ImageBuffer<image::Rgba<u8>, Vec<u8>> = default_img.clone();
    let mut img_height : u32 = 0;
    let mut img_width  : u32 = 0;

    let satellite_tle : &str = "common/tle2.txt";

    // Observer (gateway) data
    let gateway_color : [u8; 4] = [255,0,0,255]; // red 0% transparent.
    let mut gateway : Observer = Observer::new(); 
    
    // First, create new thread to read in the map of earth and plot observer point(s).
    let img_handle = thread::spawn(move || {
        let map_data   = load_map(earth_file).unwrap();
        img        = map_data.0;
        
        let (x,y) = gimme_point(&gateway.geodetic_coordinates.longitude, 
                                          &gateway.geodetic_coordinates.latitude, 
                                          &map_data.1, 
                                          &map_data.2).unwrap_or((0,0));
        assert_ne!((x,y),(0,0));

        let pix_vec = get_pixel_vector(&map_data.1, 
                                                        &map_data.2, 
                                                        &(x, y), 
                                                        false).unwrap_or(Vec::new());
        assert_ne!(pix_vec.len(), 0);

        for pixel in &pix_vec { 
           assert!(color_pixel(&mut img, pixel, &gateway_color).is_ok());
        }

        img
    });
    
    // Parallel to Earth image reading / plotting observer.
    let file = File::open(satellite_tle).unwrap();
    let reader = BufReader::new(file);
    let mut tle_string = String::from("");
    
    // This is very specific to the formatting of the current TLE files.
    for line in reader.lines() { 
        tle_string.push_str(line.unwrap().as_str());
        tle_string.push_str("\n");
    }
    
    let mut satellite_vector: Vec<Satellite> = Vec::new();
    let satellite_elements = parse_3les(&tle_string).unwrap();

    for element in satellite_elements { 
        let satellite_state: Satellite = Satellite::new(element);
        satellite_vector.push(satellite_state);
    }
    
    img = img_handle.join().unwrap_or(default_img.clone());

    Ok(())
}

/// load_map: loads a map into memory and creates a new map for editing.
/// input: filename
/// output: new rgbaimage, height, width
fn load_map(filename : &str) -> anyhow::Result<(image::RgbaImage, u32, u32)> { 
    let input_image = image::open(filename).unwrap();
    let (width, height) = input_image.dimensions();
    let mut img: image::RgbaImage = image::ImageBuffer::new(width, height);
    
    // Create the output map.
    // TODO: parallelize this
    for x_iter in 0..input_image.width() { 
        for y_iter in 0..input_image.height() {
            let color = input_image.get_pixel(x_iter,y_iter);
            
            img.put_pixel(x_iter,y_iter,color);
        }
    }
   Ok((img, height, width))
}

/// gimme_point:
/// input: lat, long, height, width
/// output: lat/long normalized x,y pixel point.
/// TODO: Wrap return in Result
fn gimme_point(lon : &f64, lat : &f64, height: &u32, width: &u32) -> anyhow::Result<(u32, u32)> { 
    let h = *height as f64;
    let w = *width as f64;
    let x = (w as f64 * (180. + lat) / 360.).floor() as u32; // row
    let y = (h as f64 * (90. - lon) / 180.).floor() as u32;   // col
    Ok((x,y))
}

/// get_pixel_vector: 
/// input: height, width, where we want the pixel centered on, is_sat flag
/// output: vector containing pixel locations
/// Note: I hate this function, I need to spend more time thinking about it.
/// Fat pixel Ex: 
///             ***
///             ***
///             ***
pub fn get_pixel_vector(height: &u32, width: &u32, center_pixel: &(u32, u32), is_sat: bool) -> anyhow::Result<Vec<(u32, u32)>> {
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
        pixel_vector.push(bot);
        pixel_vector.push(left);
        pixel_vector.push(right); 
        pixel_vector.push(top_right);
        pixel_vector.push(top_left);
        pixel_vector.push(bot_right);
        pixel_vector.push(bot_left);
    }
    Ok(pixel_vector)
}

/// color_pixel: colors a pixel in the provided image reference.
/// input: image, pixel_coordinates, color
/// output: none
pub fn color_pixel(image: &mut image::RgbaImage, pixel_coordinates: &(u32, u32), color: &[u8; 4]) -> anyhow::Result<()>{ 
    let (width, height) = image.dimensions();
    if pixel_coordinates.0 >= width { 
        anyhow::bail!("x coordinate pixel outside range of image");
    }
    if pixel_coordinates.1 >= height { 
        anyhow::bail!("y coord pixel outside bounds of image.");
    }
    image.put_pixel(pixel_coordinates.0, pixel_coordinates.1, image::Rgba(*color));
    
    Ok(())
}