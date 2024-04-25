use image::RgbImage;
use rand::{ thread_rng, Rng };
use show_image::{ create_window, event };

use crate::{ global_data::GlobalData, individual::Individual };

fn combine_images(img1: &RgbImage, img2: &RgbImage) -> RgbImage {
    let (width1, height) = img1.dimensions();
    let (width2, _) = img2.dimensions();
    let combined_width = width1 + width2;

    let mut combined_image = RgbImage::new(combined_width, height);

    // Copy pixels from the first image
    for (x, y, pixel) in img1.enumerate_pixels() {
        combined_image.put_pixel(x, y, *pixel);
    }

    // Copy pixels from the second image, starting from the end of the first
    for (x, y, pixel) in img2.enumerate_pixels() {
        let combined_x = x + width1;
        combined_image.put_pixel(combined_x, y, *pixel);
    }

    combined_image
}

#[allow(dead_code)]
pub fn show(image: &RgbImage) {
    // Create a window and display the image.
    let window = create_window("Debug", Default::default()).unwrap();
    window.set_image("image-001", image.clone()).unwrap();

    // Print keyboard events until Escape is pressed, then exit.
    // If the user closes the window, the channel is closed and the loop also exits.
    for event in window.event_channel().unwrap() {
        if let event::WindowEvent::KeyboardInput(event) = event {
            println!("{:#?}", event);
            if
                event.input.key_code == Some(event::VirtualKeyCode::Escape) &&
                event.input.state.is_pressed()
            {
                break;
            }
        }
    }
}

#[allow(dead_code)]
pub fn show_with_data(image: &RgbImage, individual: &Individual, global_data: &GlobalData) {
    // Create a window and display the image.
    let multi_fitness = individual.get_objectives();
    let fitness = individual.get_fitness();
    let title = format!(
        "Edge Value Fitness: {} Connectivity Fitness: {} Overall Deviation Fitness: {} Weighted Fitness: {}",
        multi_fitness.0,
        multi_fitness.1,
        multi_fitness.2,
        fitness
    );

    let black_white_image = individual.get_segment_border_image(global_data);
    let segment_image = individual.get_segments_image(global_data);
    let window = create_window(title, Default::default()).unwrap();
    let combined = combine_images(&black_white_image, &segment_image);
    window.set_image("image-001", combine_images(&image.clone(), &combined)).unwrap();

    // Print keyboard events until Escape is pressed, then exit.
    // If the user closes the window, the channel is closed and the loop also exits.
    for event in window.event_channel().unwrap() {
        if let event::WindowEvent::KeyboardInput(event) = event {
            println!("{:#?}", event);
            if
                event.input.key_code == Some(event::VirtualKeyCode::Escape) &&
                event.input.state.is_pressed()
            {
                break;
            }
        }
    }
}

pub fn get_edge_weighted_random_pixel_index(global_data: &GlobalData) -> usize {
    // Pick a random number between 0 and 1
    let random_number = thread_rng().gen_range(0.0..=1.0);

    // Accumulate weights until exceeding the random number
    let mut current_sum = 0.0;
    for (index, weight) in global_data.pixel_weights.iter().enumerate() {
        current_sum += *weight;
        if current_sum >= random_number {
            return index;
        }
    }

    // In case of rounding errors, return the last pixel
    (global_data.width * global_data.height - 1) as usize
}
