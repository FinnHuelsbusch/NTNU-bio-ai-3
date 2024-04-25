use std::{ collections::{ HashMap, HashSet } };

use image::{ GenericImageView, Rgb, RgbImage };
use imageproc::drawing::Canvas;
use queues::{ queue, IsQueue, Queue };
use rand::{ thread_rng, Rng };

use crate::{
    config::{ self, Config },
    distance::{ self, euclidean_distance },
    global_data::{ self, GlobalData },
    individual::{ Connection, Genome, Individual },
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

fn is_pixel_within_variance(
    pixel: &(f64, f64, f64),
    mean: &(f64, f64, f64),
    variance: &(f64, f64, f64)
) -> bool {
    // Calculate squared deviations from the mean for the new pixel
    let diff_red = (pixel.0 - mean.0).abs();
    let diff_green = (pixel.1 - mean.1).abs();
    let diff_blue = (pixel.2 - mean.2).abs();

    diff_red <= variance.0 && diff_green <= variance.1 && diff_blue <= variance.2

    // diff_red + diff_green + diff_blue <=
    //     threshold * variance.0 + threshold * variance.1 + threshold * variance.2
}

fn connect_similar_pixels(
    start_index: usize,
    child: &mut Individual,
    seen_pixels: &mut Vec<usize>,
    mean: &(f64, f64, f64),
    variance: &(f64, f64, f64),
    global_data: &GlobalData,
    max_depth: usize
) {
    let mut pixel_queue: Queue<usize> = queue![start_index];
    let mut skipped_pixels: Vec<usize> = vec![];
    let mut changed_pixels: Vec<usize> = vec![];

    // let mut copy = global_data.rgb_image.clone();

    while max_depth != seen_pixels.len() && pixel_queue.size() > 0 {
        // println!("{} {}", max_depth, seen_pixels.len());
        let current_pixel_index = pixel_queue.remove().unwrap();
        if seen_pixels.contains(&current_pixel_index) {
            continue;
        }
        seen_pixels.push(current_pixel_index);

        let column = (current_pixel_index % global_data.width) as i32;
        let row = (current_pixel_index / global_data.width) as i32;

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

            let column_new = (new_index % global_data.width) as i32;
            let row_new = (new_index / global_data.width) as i32;

            if seen_pixels.contains(&new_index) {
                continue;
            }

            let current_pixel = global_data.rgb_image.get_pixel(column_new as u32, row_new as u32);
            if
                is_pixel_within_variance(
                    &(
                        current_pixel.0[0] as f64,
                        current_pixel.0[1] as f64,
                        current_pixel.0[2] as f64,
                    ),
                    mean,
                    variance
                )
            {
                // if the pixel is similar. Redirect it to the current pixel
                child.genome[new_index] = position.2;
                pixel_queue.add(new_index).unwrap();
                changed_pixels.push(new_index);
                // let pixel = copy.get_pixel_mut(column_new as u32, row_new as u32);
                // *pixel = Rgb([position.3.0, position.3.1, position.3.2]);
            } else {
                skipped_pixels.push(new_index);
            }
        }
    }

    // println!("changed {:?}", changed_pixels.len());
    // println!("skipped {:?}", skipped_pixels.len());

    for pixel_index in skipped_pixels {
        let column = (pixel_index % global_data.width) as i32;
        let row = (pixel_index / global_data.width) as i32;

        for position in [
            (-1 as i32, 0 as i32, Connection::Up, (255 as u8, 0 as u8, 255 as u8)),
            (0, -1, Connection::Left, (0 as u8, 0 as u8, 255 as u8)),
            (0, 1, Connection::Right, (0 as u8, 255 as u8, 0 as u8)),
            (1, 0, Connection::Down, (255 as u8, 0 as u8, 0 as u8)),
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

            let y_offset = position.0;
            let x_offset = position.1;

            let new_index = ((row + y_offset) * (global_data.width as i32) +
                (column + x_offset)) as usize;

            if changed_pixels.contains(&new_index) {
                child.genome[pixel_index] = position.2;
                // let pixel = copy.get_pixel_mut(column as u32, row as u32);
                // *pixel = Rgb([position.3.0, position.3.1, position.3.2]);
            }
        }
    }

    // show(&copy);
}

pub fn destroy_small_segments(
    individual: &mut Individual,
    global_data: &GlobalData,
    minimum_coverage_percentage: f64
) {
    let segment_map = individual.get_cluster_map(
        global_data.width as i64,
        global_data.height as i64
    );

    let mut segmentation_size_map: HashMap<usize, (u32, usize, usize)> = HashMap::new();

    for row in 0..global_data.height {
        for column in 0..global_data.width {
            segmentation_size_map
                .entry(segment_map[row][column])
                .and_modify(|segment_size| {
                    segment_size.0 += 1;
                })
                .or_insert((1, row, column));
        }
    }
    // println!("Debugging startet");
    // println!("Number of segments: {}", segmentation_size_map.len());

    for (key, value) in segmentation_size_map.iter() {
        let count = value.0;
        let mut row = value.1;
        let mut column = value.2;
        let percentage_covered_by_cluster =
            (count as f64) / ((global_data.width * global_data.height) as f64);
        if percentage_covered_by_cluster < minimum_coverage_percentage {
            let mut seen_pixels: HashSet<(usize, usize)> = HashSet::new();
            let mut current_segment = segment_map[row as usize][column as usize];
            let mut current_direction = individual.genome[row * global_data.width + column];
            while
                current_segment == *key &&
                current_direction != Connection::None &&
                !seen_pixels.contains(&(row, column))
            {
                // check if current pixel is pointing outside of the picture
                if
                    (row == 0 && current_direction == Connection::Up) ||
                    (row == (global_data.height as usize) - 1 &&
                        current_direction == Connection::Down) ||
                    (column == 0 && current_direction == Connection::Left) ||
                    (column == (global_data.width as usize) - 1 &&
                        current_direction == Connection::Right)
                {
                    // the pixel is pointing outside of the picture -> it is the root of the segment
                    // check if segment can be combined with oghter segment
                    // this is not the case in the following case:
                    //  ---------
                    // |None|Left|
                    // |Up  |    |
                    // No matter how non is flipped, it will not unify the segment with any other segment
                    // println!("Pixel is pointing outside of the picture");
                    break;
                } else {
                    // the current pixel is pointing to another pixel
                    // get the new pixel
                    let new_row =
                        (row as i32) +
                        (match current_direction {
                            Connection::Up => -1,
                            Connection::Down => 1,
                            _ => 0,
                        });
                    let new_column =
                        (column as i32) +
                        (match current_direction {
                            Connection::Left => -1,
                            Connection::Right => 1,
                            _ => 0,
                        });

                    let new_pixel_segment = segment_map[new_row as usize][new_column as usize];
                    if new_pixel_segment == current_segment {
                        // the new pixel is part of the same segment
                        // set the new pixel as seen
                        seen_pixels.insert((row, column));
                        // set the new pixel as the current pixel
                        row = new_row as usize;
                        column = new_column as usize;
                        current_segment = new_pixel_segment;
                        current_direction = individual.genome[row * global_data.width + column];
                    } else {
                        panic!("This should not happen");
                    }
                }
            }

            let mut thread_rng = rand::thread_rng();
            let new_direction = match thread_rng.gen_range(1..5) {
                1 => Connection::Up,
                2 => Connection::Down,
                3 => Connection::Left,
                4 => Connection::Right,
                _ => panic!("Invalid connection value"),
            };
            // walk in chosen direction until the end of the segment is reached or the picture is left
            while segment_map[row][column] == *key {
                individual.genome[row * global_data.width + column] = new_direction;
                row = ((row as i32) +
                    (match new_direction {
                        Connection::Up => -1,
                        Connection::Down => 1,
                        _ => 0,
                    })) as usize;
                column = ((column as i32) +
                    (match new_direction {
                        Connection::Left => -1,
                        Connection::Right => 1,
                        _ => 0,
                    })) as usize;

                if row >= global_data.height || column >= global_data.width {
                    // println!("Reached end of picture");
                    break;
                }
            }
        }
    }
}

// pick a random pixel. and it all similar ones recursive without depth limit
pub fn eat_similar(child: &mut Individual, percent_of_picture: f64, global_data: &GlobalData) {
    let random_index = get_edge_weighted_random_pixel_index(global_data);

    // let random_index = 34000;

    let max_depth = (
        (global_data.width as f64) *
        (global_data.height as f64) *
        percent_of_picture
    ).round() as usize;

    // let mut test = global_data.rgb_image.clone();

    let column = (random_index % global_data.width) as i32;
    let row = (random_index / global_data.width) as i32;

    let pixel = global_data.rgb_image.get_pixel(column as u32, row as u32);
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

            // Update mean
            mean_pixel_color.0 += current_pixel.0[0] as f64;
            mean_pixel_color.1 += current_pixel.0[1] as f64;
            mean_pixel_color.2 += current_pixel.0[2] as f64;

            number_of_pixels_in_segment += 1;
        }
    }

    // calculate mean pixel color
    mean_pixel_color.0 /= number_of_pixels_in_segment as f64;
    mean_pixel_color.1 /= number_of_pixels_in_segment as f64;
    mean_pixel_color.2 /= number_of_pixels_in_segment as f64;

    let mut variance_pixel_color = (0.0, 0.0, 0.0);

    // loop over every pixel of that segment again to calculate variance
    for y in 0..global_data.height {
        for x in 0..global_data.width {
            let current_segment = segment_map[y][x];
            if current_segment != segment {
                continue;
            }

            let current_pixel = global_data.rgb_image.get_pixel(x as u32, y as u32);

            // calculate squared differences from mean
            let diff_r = ((current_pixel.0[0] as f64) - mean_pixel_color.0).powi(2);
            let diff_g = ((current_pixel.0[1] as f64) - mean_pixel_color.1).powi(2);
            let diff_b = ((current_pixel.0[2] as f64) - mean_pixel_color.2).powi(2);

            variance_pixel_color.0 += diff_r;
            variance_pixel_color.1 += diff_g;
            variance_pixel_color.2 += diff_b;
        }
    }

    // divide by the number of pixels to get the variance
    variance_pixel_color.0 /= number_of_pixels_in_segment as f64;
    variance_pixel_color.1 /= number_of_pixels_in_segment as f64;
    variance_pixel_color.2 /= number_of_pixels_in_segment as f64;

    // let mean = (pixel.0[0] as f64, pixel.0[1] as f64, pixel.0[2] as f64);
    let variance_random = thread_rng().gen_range(35.0..80.0);
    let variance = (
        variance_pixel_color.0.clamp(1.0, variance_random),
        variance_pixel_color.1.clamp(1.0, variance_random),
        variance_pixel_color.2.clamp(1.0, variance_random),
    );

    let mut mean = (mean_pixel_color.0, mean_pixel_color.1, mean_pixel_color.2);

    //initial pixel has a too high varianze to the mean, so the segment is probably to big and faulty
    if
        (mean.0 - (pixel.0[0] as f64)).abs() >= 30.0 &&
        (mean.1 - (pixel.0[1] as f64)).abs() >= 30.0 &&
        (mean.2 - (pixel.0[2] as f64)).abs() >= 30.0
    {
        mean = (pixel.0[0] as f64, pixel.0[1] as f64, pixel.0[2] as f64);
    }

    // println!("Mean: {:?}", mean);
    // println!("Variance: {:?}", variance);

    // test pixel mean
    // let pixel = global_data.rgb_image.get_pixel(column as u32, row as u32);

    connect_similar_pixels(
        random_index,
        child,
        &mut vec![],
        &mean,
        &variance,
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
                    [(pixel_y_offset + 3) as usize][(pixel_x_offset + 3) as usize];
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
                "destroy_small_segments" => {
                    destroy_small_segments(
                        child,
                        global_data,
                        mutation_config.minimum_coverage_percentage.unwrap()
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
