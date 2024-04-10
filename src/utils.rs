use image::RgbImage;
use show_image::{ create_window, event };

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
