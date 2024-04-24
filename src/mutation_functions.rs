use std::collections::HashMap;

use image::{ GenericImageView, Rgb, RgbImage };
use imageproc::drawing::Canvas;
use rand::{ thread_rng, Rng };

use crate::{
    config::{ self, Config },
    distance::euclidean_distance,
    global_data::{ self, GlobalData },
    individual::{ is_border_pixel, Connection, Genome, Individual },
    population::Population,
    utils::{ get_edge_weighted_random_pixel_index, show },
};

fn get_biggest_segment_direction(
    index: usize,
    child: &mut Individual,
    global_data: &GlobalData,
    inverse: bool
) -> Connection {
    let segmentation_map = child.get_cluster_map(
        global_data.width as i64,
        global_data.height as i64
    );

    let mut segmentation_size_map: HashMap<usize, u32> = HashMap::new();

    for y in 0..global_data.height {
        for x in 0..global_data.width {
            segmentation_size_map
                .entry(segmentation_map[y][x])
                .and_modify(|segment_size| {
                    *segment_size += 1;
                })
                .or_insert(1);
        }
    }

    let mut highest = 0;
    let mut highest_direction = Connection::None;
    let mut lowest = 0xffffffff;
    let mut lowest_direction = Connection::None;

    let column = (index % global_data.width) as i32;
    let row = (index / global_data.width) as i32;

    for position in [
        (-1 as i32, 0 as i32, Connection::Up),
        (0, -1, Connection::Left),
        (0, 1, Connection::Right),
        (1, 0, Connection::Down),
    ] {
        if
            (row == 0 && position.0 == -1) ||
            (row == (global_data.height as i32) - 1 && position.0 == 1) ||
            (column == 0 && position.1 == -1) ||
            (column == (global_data.width as i32) - 1 && position.1 == 1)
        {
            continue;
        }

        let segment_id =
            segmentation_map[(row + position.0) as usize][(column + position.1) as usize];
        let number_of_segments = *segmentation_size_map.get(&segment_id).unwrap();

        if number_of_segments > highest {
            highest = number_of_segments;
            highest_direction = position.2;
        }

        if number_of_segments < lowest {
            lowest = number_of_segments;
            lowest_direction = position.2;
        }
    }

    return if inverse { lowest_direction } else { highest_direction };
}

fn flip_to_biggest_segment(child: &mut Individual, global_data: &GlobalData) {
    let index = get_edge_weighted_random_pixel_index(global_data);

    let highest_direction = get_biggest_segment_direction(index, child, global_data, false);

    child.genome[index] = highest_direction;
}

fn flip_to_smallest_segment(child: &mut Individual, global_data: &GlobalData) {
    let index = get_edge_weighted_random_pixel_index(global_data);

    let lowest_direction = get_biggest_segment_direction(index, child, global_data, true);

    child.genome[index] = lowest_direction;
}

fn connect_similar_pixels_recursive(
    index: usize,
    child: &mut Individual,
    seen_pixels: &mut Vec<usize>,
    mean: &Rgb<u8>,
    distance_threshold: f64,
    global_data: &GlobalData,
    max_depth: usize
) {
    if max_depth == seen_pixels.len() {
        return;
    }

    seen_pixels.push(index);

    let column = (index % global_data.width) as i32;
    let row = (index / global_data.width) as i32;

    // Go trough every neighbor
    for position in [
        (-1 as i32, 0 as i32, Connection::Down, (255 as u8, 0 as u8, 0 as u8)),
        (0, -1, Connection::Right, (0 as u8, 255 as u8, 0 as u8)),
        (0, 1, Connection::Left, (0 as u8, 0 as u8, 255 as u8)),
        (1, 0, Connection::Up, (255 as u8, 0 as u8, 255 as u8)),
    ] {
        // ignore edges outside of the picture
        if
            (row == 0 && position.0 == -1) ||
            (row == (global_data.height as i32) - 1 && position.0 == 1) ||
            (column == 0 && position.1 == -1) ||
            (column == (global_data.width as i32) - 1 && position.1 == 1)
        {
            continue;
        }

        // calculate the index in the genome based on the position
        let y_offset = position.0;
        let x_offset = position.1;

        let new_index = ((row + y_offset) * (global_data.width as i32) +
            (column + x_offset)) as usize;

        let column_new = (index % global_data.width) as i32;
        let row_new = (index / global_data.width) as i32;

        if seen_pixels.contains(&new_index) {
            continue;
        }

        let euclidian_distance = euclidean_distance(
            global_data.rgb_image.get_pixel(column_new as u32, row_new as u32),
            mean
        );

        if euclidian_distance <= distance_threshold {
            // if the pixel is similar. Redirect it to the current pixel

            child.genome[new_index] = position.2;

            // and go recursive inside it
            connect_similar_pixels_recursive(
                new_index,
                child,
                seen_pixels,
                mean,
                distance_threshold,
                global_data,
                max_depth
            );
        }
    }
}

// pick a random pixel. and it all similar ones recursive without depth limit
pub fn eat_similar(child: &mut Individual, percent_of_picture: f64, global_data: &GlobalData) {
    let random_index = thread_rng().gen_range(0..child.genome.len());

    let max_depth = (
        (global_data.width as f64) *
        (global_data.height as f64) *
        percent_of_picture
    ).round() as usize;

    // let mut test = global_data.rgb_image.clone();

    let column = (random_index % global_data.width) as i32;
    let row = (random_index / global_data.width) as i32;

    // let pixel = test.get_pixel_mut(column as u32, row as u32);
    // *pixel = Rgb([0, 255, 255]);

    // get the segment from the pixel
    let segment_map = child.get_cluster_map(global_data.width as i64, global_data.height as i64);
    let segment = segment_map[row as usize][column as usize];

    let mut mean_pixel_color = (0.0, 0.0, 0.0);
    let mut number_of_pixels_in_segment = 0;

    // loop over every pixel of that segment
    for y in 0..global_data.height {
        for x in 0..global_data.width {
            let current_segment = segment_map[y][x];
            if current_segment != segment {
                continue;
            }

            let current_pixel = global_data.rgb_image.get_pixel(x as u32, y as u32);
            mean_pixel_color.0 += current_pixel.0[0] as f64;
            mean_pixel_color.1 += current_pixel.0[1] as f64;
            mean_pixel_color.2 += current_pixel.0[2] as f64;
            number_of_pixels_in_segment += 1;
        }
    }

    mean_pixel_color.0 /= number_of_pixels_in_segment as f64;
    mean_pixel_color.1 /= number_of_pixels_in_segment as f64;
    mean_pixel_color.2 /= number_of_pixels_in_segment as f64;

    connect_similar_pixels_recursive(
        random_index,
        child,
        &mut vec![],
        &Rgb([
            mean_pixel_color.0.round() as u8,
            mean_pixel_color.1.round() as u8,
            mean_pixel_color.2.round() as u8,
        ]),
        40.0,
        global_data,
        max_depth
    );

    // show(&test)
}

fn flip_one_bit(child: &mut Individual, global_data: &GlobalData) {
    let index = get_edge_weighted_random_pixel_index(global_data);
    let new_connection = match rand::thread_rng().gen_range(0..5) {
        0 => Connection::None,
        1 => Connection::Up,
        2 => Connection::Down,
        3 => Connection::Left,
        4 => Connection::Right,
        _ => panic!("Invalid connection value"),
    };
    child.genome[index] = new_connection;
}

fn flip_to_smallest_deviation(child: &mut Individual, global_data: &GlobalData, radius: usize) {
    // Cant use radius 0. Because it would not look up anything
    assert_ne!(radius, 0);

    let index = get_edge_weighted_random_pixel_index(global_data);

    let mut smallest_deviation = f64::INFINITY;
    let mut smallest_direction = Connection::None;

    for position in [
        (-1 as i32, 0 as i32, Connection::Up),
        (0, -1, Connection::Left),
        (0, 1, Connection::Right),
        (1, 0, Connection::Down),
    ] {
        let column = (index % global_data.width) as i32;
        let row = (index / global_data.width) as i32;

        let mut direction_deviation = f64::INFINITY;

        for radius in 1..=radius as i32 {
            let pixel_y_offset = position.0 * (radius as i32);
            let pixel_x_offset = position.1 * (radius as i32);

            if
                (row - radius < 0 && pixel_y_offset != 0) ||
                (row + radius > (global_data.height as i32) - 1 && pixel_y_offset != 0) ||
                (column - radius < 0 && pixel_x_offset != 0) ||
                (column + radius > (global_data.width as i32) - 1 && pixel_x_offset != 0)
            {
                continue;
            }

            direction_deviation +=
                global_data.euclidean_distance_map[row as usize][column as usize]
                    [(pixel_y_offset as usize) + (radius as usize)]
                    [(pixel_x_offset as usize) + (radius as usize)];
        }

        direction_deviation /= radius as f64;

        if direction_deviation < smallest_deviation {
            smallest_deviation = direction_deviation;
            smallest_direction = position.2;
        }
    }

    child.genome[index] = smallest_direction;
}

pub fn mutate(population: &mut Population, config: &Config, global_data: &GlobalData) {
    let mut rng = rand::thread_rng();
    for mutation_config in config.mutations.iter() {
        // Calculate the number of crossovers which should happen for the specific config
        let number_of_mutations: u64 = (
            (config.population_size as f64) * mutation_config.probability.unwrap()
        ).ceil() as u64;

        for _ in 0..number_of_mutations {
            let individual_index: usize = rng.gen_range(0..config.population_size);
            let child = &mut population[individual_index];
            match mutation_config.name.as_str() {
                "flip_one_bit" => {
                    flip_one_bit(child, global_data);
                }
                "flip_to_smallest_segment" => {
                    flip_to_smallest_segment(child, global_data);
                }
                "flip_to_biggest_segment" => {
                    flip_to_biggest_segment(child, global_data);
                }
                "eat_similar" => {
                    eat_similar(
                        child,
                        mutation_config.max_depth_percent_of_picture.unwrap(),
                        global_data
                    );
                }
                "flip_to_smallest_deviation" => {
                    flip_to_smallest_deviation(
                        child,
                        global_data,
                        mutation_config.radius.unwrap_or(1)
                    );
                }
                _ =>
                    panic!(
                        "Didn't have an Implementation for mutation function: {:?}",
                        mutation_config.name.as_str()
                    ),
            }

            population[individual_index].set_needs_update();
        }
    }
}
