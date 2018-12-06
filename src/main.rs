extern crate image;

use std::path::{PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use image::{DynamicImage, Rgb, RgbImage};

const IMAGE_IN: &'static str = "crystalized.jpg";

fn main() {
    let start_time = Instant::now();
    let mut outliner = Outliner::new(IMAGE_IN, &start_time);

    // Split up the processing into quarters
    let x_bounds = [outliner.new_image.width() / 2, outliner.new_image.width()];
    let y_bounds = [outliner.new_image.height() / 2, outliner.new_image.height()];

    // Top left starts at 0, 0 travels +1, +1
    for x in 0..x_bounds[0] {
        for y in 0..y_bounds[0] {
            outliner.draw_outline(x, y, &[(0, 1), (1, 0), (1, 1)]);
        }
    }
    println!("25% Done: {}", current_time(&start_time));

    // Bottom left starts at 0, height travels +1, -1
    for x in 0..x_bounds[0] {
        for y in ((y_bounds[0] - 1)..y_bounds[1]).rev() {
            outliner.draw_outline(x, y, &[(0, -1), (1, 0), (1, -1)]);
        }
    }
    println!("50% Done: {}", current_time(&start_time));

    // Top right starts at width, 0 travels -1, 1
    for x in ((x_bounds[0] - 1)..x_bounds[1]).rev() {
        for y in 0..y_bounds[0] {
            outliner.draw_outline(x, y, &[(0, 1), (-1, 0), (-1, 1)]);
        }
    }
    println!("75% Done: {}", current_time(&start_time));

    // Bottom right starts at width, height travels -1, -1
    for x in ((x_bounds[0] - 1)..x_bounds[1]).rev() {
        for y in ((y_bounds[0] - 1)..y_bounds[1]).rev() {
            outliner.draw_outline(x, y, &[(0, -1), (-1, 0), (-1, -1)]);
        }
    }
    println!("All Done: {}", current_time(&start_time));

    outliner.save();
}

/// Figures out the luminance of a pixel
fn pixel_lumin(pixel: &[u8; 3]) -> f32 {
    let red = (pixel[0] as f32) / 255.0;
    let green = (pixel[1] as f32) / 255.0;
    let blue = (pixel[2] as f32) / 255.0;
    return 0.2126 * red + 0.7152 * green + 0.0722 * blue;
}
fn current_time(start_time: &Instant) -> String {
    let diff = start_time.elapsed();
    let seconds = diff.as_secs();
    let millis = diff.subsec_millis();
    let micros = diff.subsec_micros() % 1_000;
    let nanos = diff.subsec_nanos() % 1_000;

    format!("{}s {}ms {}μs {}ns", seconds, millis, micros, nanos)
}

struct Outliner {
    file_name: PathBuf,
    new_image: RgbImage,
    lumin_map: Vec<f32>,
    start_time: Instant,
}
impl Outliner {
    fn new(file_name: &str, start_time: &Instant) -> Outliner {
        let image_in = PathBuf::from(file_name);
        let base_image = {
            let image = image::open(&image_in)
                .expect("Failed to read in the image");
            match image {
                DynamicImage::ImageRgb8(i) => i,
                _ => image.to_rgb(),
            }
        };
        let (width, height) = (base_image.width(), base_image.height());
        println!("Loaded image ({}x{}): {}", width, height, current_time(&start_time));

        // Make a flat vector for an easier time on memory
        let mut lumin_map: Vec<f32> = Vec::with_capacity((width * height) as usize);
        for x in 0..width {
            for y in 0..height {
                lumin_map.push(pixel_lumin(&base_image.get_pixel(x, y).data));
            }
        }
        println!("Done with the luminance map: {}", current_time(&start_time));

        Outliner {
            file_name: image_in,
            // Just overwrite our original image since every pixel will get changed
            new_image: base_image,
            lumin_map,
            start_time: start_time.clone(),
        }
    }

    /// Gets the lumin at the x and y coords from the lumin map
    fn get_lumin(&self, x: u32, y: u32) -> f32 {
        // Since the map is in flat columns of the image
        //         (0, 0)         (1, 0)
        // image: [<height_len>, <height_len>, ...]
        self.lumin_map[((x * self.new_image.height()) + y) as usize]
    }

    fn draw_outline(&mut self, x: u32, y: u32, comp_indices: &[(i32, i32)]) {
        let lumin = self.get_lumin(x, y);
        let lumin_sum = comp_indices.iter().map(|&(dx, dy)| {
            let new_x = (x as i32 + dx) as u32;
            let new_y = (y as i32 + dy) as u32;
            (lumin - self.get_lumin(new_x, new_y)).abs()
        }).sum::<f32>();

        let average_lumin = lumin_sum / comp_indices.len() as f32;
        let gray_value = ((1.0 - average_lumin.sqrt()) * 255.0) as u8;

        self.new_image.put_pixel(x, y, Rgb { data: [gray_value, gray_value, gray_value] });
    }

    fn save(self) {
        let save_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let file_stem = self.file_name.file_stem().unwrap().to_str().unwrap();
        self.new_image.save(self.file_name.with_file_name(
            format!("rustOutline-{}{}.png", file_stem, save_time)
        )).expect("Failed to save the new outline");
        println!("Done saving: {}", current_time(&self.start_time));
    }
}
