use image::RgbImage;
use rand::{ thread_rng, Rng };
use show_image::{ create_window, event };

use crate::global_data::{ self, GlobalData };

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
