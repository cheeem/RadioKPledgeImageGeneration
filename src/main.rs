use image;
use std::env;
use std::str;

fn main() {

    let mut args: env::Args = env::args();

    let cmd: String = args.next().unwrap();

    let max: u32 = match args.next().map(parse_amount) {
        Some(Some(value)) => value,
        Some(None) => return println!("Donation Goal Must Be a Number"),
        None => return println!("Please Provide 2 Additional Parameters for the Donation Goal and Current Donation Amount\nExample: {cmd} 10000 4500")
    };

    let value: u32 = match args.next().map(parse_amount) {
        Some(Some(value)) => value,
        Some(None) => return println!("Current Donation Amount Must Be a Number"),
        None => return println!("Please Provide an Additional Parameter for the Current Donation Amount\nExamples:\n{cmd} 10000 4500\n{cmd} 500 550")
    };

    let initial_file_path_owned: Option<String> = args.next();
    let final_file_path_owned: Option<String> = args.next();

    // change to correct if wrong format
    
    let fill_rgba: [u8; 4] = match args.next().map(parse_rgba) {
        Some(Some(rgba)) => rgba,
        Some(None) => return println!("Fill RGBA Values Must Be Between 0-255 and in One of the Following Formats:\n{{}},{{}},{{}} or {{}},{{}},{{}},{{}}\nExamples:\n255,40,105\n200,0,0,100"),
        None => [173, 216, 230, 255],
    };

    let edge_rgba: [u8; 4] = match args.next().map(parse_rgba) {
        Some(Some(rgba)) => rgba,
        Some(None) => return println!("Edge RGBA Values Must Be Between 0-255 and in One of the Following Formats:\n{{}},{{}},{{}} or {{}},{{}},{{}},{{}}\nExamples:\n255,40,105\n200,0,0,100"),
        None => [0, 0, 0, 255],
    };

    let initial_file_path: &str = initial_file_path_owned.as_deref().unwrap_or("Thermometer.png");
    let final_file_path: &str = final_file_path_owned.as_deref().unwrap_or("RadioKPledgeThermometer.png");

    if initial_file_path
        .rfind('.')
        .filter(|pos| &initial_file_path[*pos..] == ".png")
        .is_none()
    {
        return println!("Initial Image Must Be A PNG");
    }  

    let mut img: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> = match image::ImageReader::open(initial_file_path).map(image::ImageReader::decode) {
        Ok(Ok(image::DynamicImage::ImageRgba8(img))) => img,
        Ok(_) => return println!("{initial_file_path} Could Not Be Decoded"),
        Err(_) => return println!("{initial_file_path} Does Not Exist or Could Not Be Opened")
    };

    let width: u32 = img.width();

    let (inner_beginning_y, inner_end_y) = find_bounds(
        (0..img.height()).map(|y| (y, img.get_pixel(width / 2, y).0)), 
        edge_rgba
    );
    
    let fill_height: u32 = if value < max {
        ((inner_end_y - inner_beginning_y) * (max - value)) / max
    } else {
        println!("Current Donation Amount is Larger Than Donation Goal, Rounding Down to Fill 100% of Image");
        0
    };

    for y in (inner_beginning_y + fill_height)..inner_end_y {

        let (inner_beginning_x, inner_end_x) = find_bounds(
            (0..width).map(|x| (x, img.get_pixel(x, y).0)), 
            edge_rgba
        );

        if inner_end_x == 0 {
            continue;
        }

        for x in inner_beginning_x..inner_end_x {

            let is_transparent: bool = img.get_pixel(x, y).0[3] == 0;

            if is_transparent {
                *img.get_pixel_mut(x, y) = image::Rgba::from(fill_rgba);
            }

        }

    }

    let final_file_path: &str = match final_file_path.rfind('.').map(|pos| (&final_file_path[pos..] != ".png").then(|| pos)) {
        Some(None) => final_file_path,
        Some(Some(pos)) => &format!("{}.png", &final_file_path[..pos]),
        None => &format!("{final_file_path}.png"),
    };

    if img
        .save_with_format(final_file_path, image::ImageFormat::Png)
        .is_err() 
    {
        println!("Failed to Save {final_file_path}");
    }
    
}

fn parse_amount(str: String) -> Option<u32> {
    
    if str.chars().next()? == '$' {
        str[1..].parse().ok()
    } else {
        str.parse().ok()
    }

}

fn parse_rgba(str: String) -> Option<[u8; 4]> {
    
    let mut split: str::Split<'_, char> = str.split(',');
    
    return Some([
        split.next()?.parse().ok()?,
        split.next()?.parse().ok()?,
        split.next()?.parse().ok()?,
        split.next().map(str::parse).unwrap_or(Ok(255)).ok()?,
    ]);

}

fn find_bounds(iter: impl Iterator<Item = (u32, [u8; 4])>, edge_rgba: [u8; 4]) -> (u32, u32) {

    let mut found_edge_pixel: bool = false;
    let mut end_edge_thickness: u32 = 0;
    let mut inner_beginning_idx: u32 = 0;
    let mut inner_end_idx: u32 = 0;

    for (idx, pixel) in iter {

        let is_edge_pixel: bool = pixel == edge_rgba;

        if is_edge_pixel {
            
            found_edge_pixel = true;
            
            if inner_beginning_idx != 0 {
                end_edge_thickness += 1;
            }

            continue;

        }

        if  inner_beginning_idx == 0 && found_edge_pixel {
            inner_beginning_idx = idx;
            continue;
        }

        if end_edge_thickness != 0 {
            inner_end_idx = idx - end_edge_thickness - 1;
            end_edge_thickness = 0;
        }

    }

    return (inner_beginning_idx, inner_end_idx);

}
