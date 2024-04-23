use std::collections::HashMap;

use rand::Rng;

use crate::{
    config::{ self, Config },
    global_data::{ self, GlobalData },
    individual::{ Connection, Genome, Individual },
    population::Population,
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
    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..child.genome.len());

    let highest_direction = get_biggest_segment_direction(index, child, global_data, false);

    child.genome[index] = highest_direction;
}

fn flip_to_smallest_segment(child: &mut Individual, global_data: &GlobalData) {
    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..child.genome.len());

    let lowest_direction = get_biggest_segment_direction(index, child, global_data, true);

    child.genome[index] = lowest_direction;
}

fn flip_one_bit(child: &mut Individual) {
    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..child.genome.len());
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

    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..child.genome.len());

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
                    flip_one_bit(child);
                }
                "flip_to_smallest_segment" => {
                    flip_to_smallest_segment(child, global_data);
                }
                "flip_to_biggest_segment" => {
                    flip_to_biggest_segment(child, global_data);
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
