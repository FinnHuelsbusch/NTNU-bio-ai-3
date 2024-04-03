use std::{collections::{hash_map, HashMap, HashSet}, vec};


use rand::Rng;



// create a enum
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Connection {
    None,
    Up,
    Down,
    Left,
    Right,
}

pub type Genome = Vec<Connection>;

#[derive(Debug, Clone)]
pub struct Individual {
    pub rgb_image: image::RgbImage,
    pub genome: Genome,

    // penalty
    pub edge_value_fitness: f64,
    pub connectivity_fitness: f64,
    pub overall_deviation_fitness: f64,
    pub euclidean_distance: Vec<Vec<Vec<Vec<f64>>>>
}

impl Individual {
    pub fn new(image_path: &str) -> Individual {
        let rgb_image = Individual::open_image_as_rgb(image_path);
        // randomly choose the type of every field in the genome
        let mut genome =
            Vec::with_capacity((rgb_image.width() * rgb_image.height()).try_into().unwrap());
        for _ in 0..rgb_image.width() * rgb_image.height() {
            let value = rand::thread_rng().gen_range(0..5);
            match value {
                0 => genome.push(Connection::None),
                1 => genome.push(Connection::Up),
                2 => genome.push(Connection::Down),
                3 => genome.push(Connection::Left),
                4 => genome.push(Connection::Right),
                _ => panic!("Invalid value"),
            }
        }
        let mut individual = Individual {
            rgb_image,
            genome,
            edge_value_fitness: 0.0,
            connectivity_fitness: 0.0,
            overall_deviation_fitness: 0.0,
            euclidean_distance: Vec::new(),
        };
        individual.euclidean_distance = individual.calculate_euclidean_distance();
        individual
    }

    fn open_image_as_rgb(image_path: &str) -> image::RgbImage {
        let img = image::open(image_path).unwrap();
        img.to_rgb8()
    }

    fn do_arrows_face_each_other(arrow1: Connection, arrow2: Connection) -> bool {
        match arrow1 {
            Connection::None => false,
            Connection::Up => arrow2 == Connection::Down,
            Connection::Down => arrow2 == Connection::Up,
            Connection::Left => arrow2 == Connection::Right,
            Connection::Right => arrow2 == Connection::Left,
        }
    }

    fn get_clustered_image(&self) -> Vec<Vec<usize>> {
        // create two-dimensional vector to store the cluster
        let mut clustered_image: Vec<Vec<usize>> =
            vec![
                vec![0; self.rgb_image.height().try_into().unwrap()];
                self.rgb_image.width().try_into().unwrap()
            ];
        let mut next_unused_cluster_id = 1;
        for row in 0..self.rgb_image.height() as usize {
            for column in 0..self.rgb_image.width() as usize {
                if clustered_image[column][row] != 0 {
                    continue;
                }
                let mut current_column = column;
                let mut current_row = row;
                let mut visited_pixels: HashSet<(usize, usize)> = HashSet::new();
                let mut current_arrow = self.genome[(row * self.rgb_image.height() as usize + column) as usize];
                let mut previous_arrow = Connection::None;
                self.genome[(row * self.rgb_image.height() as usize + column) as usize];
                let mut cluster_to_use = 0;
                while current_arrow != Connection::None && !Individual::do_arrows_face_each_other(current_arrow, previous_arrow) {
                    visited_pixels.insert((current_column, current_row));
                    let mut pixel_pointed_to = (column, row);
                    match current_arrow {
                        Connection::Up => {
                            match pixel_pointed_to.1.checked_sub(1) {
                                Some(value) => pixel_pointed_to.0 = value,
                                None => break,
                            }
                        }
                        Connection::Down => {
                            match pixel_pointed_to.1.checked_add(1) {
                                Some(value) => pixel_pointed_to.0 = value,
                                None => break,
                            }
                        }
                        Connection::Left => {
                            match pixel_pointed_to.0.checked_sub(1) {
                                Some(value) => pixel_pointed_to.1 = value,
                                None => break,
                            }
                        }
                        Connection::Right => {
                            match pixel_pointed_to.0.checked_add(1) {
                                Some(value) => pixel_pointed_to.1 = value,
                                None => break,
                            }
                        }
                        _ => panic!("Invalid value"),
                    }
                    if visited_pixels.contains(&pixel_pointed_to) 
                    || pixel_pointed_to.0 >= self.rgb_image.width() as usize
                    || pixel_pointed_to.1 >= self.rgb_image.height() as usize{
                        break;
                    }
                    current_arrow = self.genome[(pixel_pointed_to.0 * self.rgb_image.height() as usize + pixel_pointed_to.1) as usize];
                    cluster_to_use = clustered_image[pixel_pointed_to.0][pixel_pointed_to.1];
                    current_column = pixel_pointed_to.0;
                    current_row = pixel_pointed_to.1;
                }
                if cluster_to_use == 0 {
                    cluster_to_use = next_unused_cluster_id;
                    next_unused_cluster_id += 1;
                }
                for visited_pixel in visited_pixels {
                    clustered_image[visited_pixel.0][visited_pixel.1] = cluster_to_use;
                }
            }
        }
        // Return the clustered_image
        clustered_image
    }

    fn calculate_edge_value(&self, clusterd_image: &Vec<Vec<usize>>) -> f64 {
        let mut edge_value = 0.0;
        // iterate over every pixel in the image
        for outer_row in 0..self.rgb_image.height() as usize {
            for outer_column in 0..self.rgb_image.width() as usize {
                // iterate through the pixels in the 3x3 neighborhood
                for inner_row in 0..3 {
                    for inner_column in 0..3 {
                        let column = match (outer_column + inner_column).checked_sub(1) {
                            Some(value) => value,
                            None => continue,
                            
                        };
                        let row = match (outer_row + inner_row).checked_sub(1) {
                            Some(value) => value,
                            None => continue,
                        };
                        // check if the pixel is within the image
                        if  column >= self.rgb_image.width() as usize
                            || row >= self.rgb_image.height() as usize
                        {
                            continue;
                        }
                        // check if the pixel is in the same cluster
                        if clusterd_image[outer_column][outer_row] == clusterd_image[column][row] {
                            continue;
                        }
                        // calculate the difference in color between the two pixels
                        let pixel_color = self.rgb_image.get_pixel(outer_column as u32, outer_row as u32);
                        let neighbor_color = self.rgb_image.get_pixel(column as u32, row as u32);
                        let pixel_color = pixel_color.0;
                        let neighbor_color = neighbor_color.0;
                        let mut color_difference = 0.0;
                        // calculate euclidean distance between the two colors
                        for i in 0..3 {
                            color_difference += (pixel_color[i] as f64 - neighbor_color[i] as f64).powi(2);
                        }
                        color_difference = color_difference.sqrt();
                        // add the color difference to the edge value
                        edge_value += color_difference;
                    }
                }
            }
        }
        edge_value
    }

        

    fn calculate_euclidean_distance(&self) -> Vec<Vec<Vec<Vec<f64>>>> {
        let mut euclidean_distance = vec![
            vec![
                vec![
                    vec![0.0; self.rgb_image.width() as usize];
                    self.rgb_image.height() as usize
                ];
                self.rgb_image.width() as usize
            ];
            self.rgb_image.height() as usize
        ];
        for outer_row in 0..self.rgb_image.height() as usize {
            for outer_column in 0..self.rgb_image.width() as usize {
                for inner_row in 0..self.rgb_image.height() as usize {
                    for inner_column in 0..self.rgb_image.width() as usize {
                        let pixel_color = self.rgb_image.get_pixel(outer_column as u32, outer_row as u32);
                        let other_pixel_color = self.rgb_image.get_pixel(inner_column as u32, inner_row as u32);
                        let red_difference = pixel_color[0].abs_diff(other_pixel_color[0]);
                        let green_difference = pixel_color[1].abs_diff(other_pixel_color[1]);
                        let blue_difference = pixel_color[2].abs_diff(other_pixel_color[2]);
                        euclidean_distance[outer_column][outer_row][inner_column][inner_row] = ((red_difference.pow(2) + green_difference.pow(2) + blue_difference.pow(2))as f64).sqrt();

                    }
                }
            }
        }
        euclidean_distance
    }




    pub fn update_objectives(&mut self) {
        let clustered_image = self.get_clustered_image();
        let mut edge_value_fitness: f64 = 0.0;
        let mut connectivity_fitness: f64 = 0.0;
        let mut overall_deviation_fitness: f64 = 0.0;
        let mut overall_deviation_map: HashMap<usize, (f64,f64,f64, u32)> = HashMap::new();
        for row in 0..self.rgb_image.height() as usize {
            for column in 0..self.rgb_image.width() as usize {
                // iterate through the pixels in the 3x3 neighborhood
                for inner_row in 0..3 {
                    for inner_column in 0..3 {
                        let column = match (column + inner_column).checked_sub(1) {
                            Some(value) => value,
                            None => continue,
                        };
                        let row = match (row + inner_row).checked_sub(1) {
                            Some(value) => value,
                            None => continue,
                        };
                        // check if the pixel is within the image
                        if column >= self.rgb_image.width() as usize
                            || row >= self.rgb_image.height() as usize
                        {
                            continue;
                        }
                        // check if the pixel is in the same cluster
                        if clustered_image[column][row] == clustered_image[column][row] {
                            continue;
                        }
                        // get edge value difference from EUCLIDEAN_DISTANCE
                        edge_value_fitness += self.euclidean_distance[column][row][column][row];
                        connectivity_fitness += 0.125;
                    }
                }
                overall_deviation_map
                            .entry(clustered_image[column][row])
                            .and_modify(|(sum_red, sum_green, sum_blue, count)| {
                                *sum_red += self.rgb_image.get_pixel(column as u32, row as u32)[0] as f64;
                                *sum_green += self.rgb_image.get_pixel(column as u32, row as u32)[1] as f64;
                                *sum_blue += self.rgb_image.get_pixel(column as u32, row as u32)[2] as f64;
                                *count += 1;
                            })
                            .or_insert((
                                self.rgb_image.get_pixel(column as u32, row as u32)[0] as f64,
                                self.rgb_image.get_pixel(column as u32, row as u32)[1] as f64,
                                self.rgb_image.get_pixel(column as u32, row as u32)[2] as f64,
                                1,
                            ));
            }
        }
        let mut average_color_per_cluster: HashMap<usize, (f64, f64, f64)> = HashMap::new();
        for (cluster_id, (sum_red, sum_green, sum_blue, count)) in overall_deviation_map {
            average_color_per_cluster.insert(
                cluster_id,
                (
                    sum_red / count as f64,
                    sum_green / count as f64,
                    sum_blue / count as f64,
                ),
            );
        }
        overall_deviation_fitness = 0.0;
        for row in 0..self.rgb_image.height() as usize {
            for column in 0..self.rgb_image.width() as usize {
                let pixel_color = self.rgb_image.get_pixel(column as u32, row as u32);
                let average_color = average_color_per_cluster.get(&clustered_image[column][row]).unwrap();
                overall_deviation_fitness += ((pixel_color[0] as f64 - average_color.0).powi(2)
                    + (pixel_color[1] as f64 - average_color.1).powi(2)
                    + (pixel_color[2] as f64 - average_color.2).powi(2))
                    .sqrt();
            }
        }
        self.edge_value_fitness = edge_value_fitness;
        self.connectivity_fitness = connectivity_fitness;
        self.overall_deviation_fitness = overall_deviation_fitness;
    }

    pub fn dominates(&self, other: &Individual) -> bool {
        self.edge_value_fitness <= other.edge_value_fitness
            && self.connectivity_fitness <= other.connectivity_fitness
            && self.overall_deviation_fitness <= other.overall_deviation_fitness
            && (self.edge_value_fitness > other.edge_value_fitness // higher fitness is better
                || self.connectivity_fitness < other.connectivity_fitness // lower fitness is better
                || self.overall_deviation_fitness < other.overall_deviation_fitness) // lower fitness is better
    }
    
}








