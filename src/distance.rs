use image::{ Rgb, RgbImage };

pub fn euclidean_distance(pixel_a: &Rgb<u8>, pixel_b: &Rgb<u8>) -> f64 {
    let red_difference = pixel_a[0].abs_diff(pixel_b[0]) as u64;
    let green_difference = pixel_a[1].abs_diff(pixel_b[1]) as u64;
    let blue_difference = pixel_a[2].abs_diff(pixel_b[2]) as u64;

    return (
        (red_difference.pow(2) + green_difference.pow(2) + blue_difference.pow(2)) as f64
    ).sqrt();
}

pub fn calculate_euclidean_distance_map_for_neighbors(
    rgb_image: RgbImage
) -> Vec<Vec<Vec<Vec<f64>>>> {
    let height = rgb_image.height();
    let width = rgb_image.width();
    let mut euclidean_distance_map =
        vec![
        vec![
            vec![
                vec![0.0; 3];
                3
            ];
            width as usize
        ];
        height as usize
    ];

    for row in 0..height as usize {
        for column in 0..width as usize {
            let current_pixel = rgb_image.get_pixel(column as u32, row as u32);
            for x_offset in -1 as i32..=1 {
                for y_offset in -1 as i32..=1 {
                    // Dont calculate boundaries
                    if
                        (row == 0 && y_offset == -1) ||
                        ((row as u32) == height - 1 && y_offset == 1) ||
                        (column == 0 && x_offset == -1) ||
                        ((column as u32) == width - 1 && x_offset == 1)
                    {
                        continue;
                    }

                    let neighbor_pixel = rgb_image.get_pixel(
                        ((column as i32) + x_offset) as u32,
                        ((row as i32) + y_offset) as u32
                    );

                    euclidean_distance_map[row][column][(y_offset + 1) as usize][
                        (x_offset + 1) as usize
                    ] = euclidean_distance(current_pixel, neighbor_pixel);
                }
            }
        }
    }
    euclidean_distance_map
}

/**
 * Takes a Pixel position and an offset and gets the nearest pixel value according to the exercise description
 */
pub fn get_nearest_neighbor_value(x_offset: i32, y_offset: i32) -> i32 {
    let tuple = (x_offset, y_offset);
    match tuple {
        (-1, -1) => 7,
        (0, -1) => 3,
        (1, -1) => 5,
        (-1, 0) => 2,
        (0, 0) => 0,
        (1, 0) => 1,
        (-1, 1) => 8,
        (0, 1) => 4,
        (1, 1) => 6,
        _ => panic!(),
    }
}
