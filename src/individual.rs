use std::{ collections::HashMap, vec };
use image::RgbImage;
use rand::Rng;

use crate::{ distance::get_nearest_neighbor_value, utils::show };

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
fn get_connected_pixels_for_pixel(
    genome: &Genome,
    index: i64,
    width: i64,
    seen_pixels: &mut Vec<i64>
) -> Vec<i64> {
    if seen_pixels.contains(&index) {
        return vec![];
    }

    seen_pixels.push(index);
    let mut connected_pixels = vec![index];
    // check direction going to and follow the path
    let direction = genome[index as usize];
    match direction {
        Connection::None => {}
        Connection::Up => {
            // if the direction is up the index needs to be at least in the second row. So width should be at least one time in the index
            if index - width > 0 {
                connected_pixels.append(
                    &mut get_connected_pixels_for_pixel(genome, index - width, width, seen_pixels)
                );
            }
        }
        Connection::Down => {
            // if the direction is down the index needs to be at least in the second to last row. So index + width should not be higher than genome length
            if index + width < (genome.len() as i64) {
                connected_pixels.append(
                    &mut get_connected_pixels_for_pixel(genome, index + width, width, seen_pixels)
                );
            }
        }
        Connection::Left => {
            // if the direction is left the index needs bigger than the wrapping of the width. So index % width should give the index in a row and this needs to be bigger than 0
            if index % width > 0 {
                connected_pixels.append(
                    &mut get_connected_pixels_for_pixel(genome, index - 1, width, seen_pixels)
                );
            }
        }
        Connection::Right => {
            // if the direction is right the index needs to be less than the wrapping of the width. So index % width should give the index in a row and this needs to be less than width - 1 (width is the edge)
            if index % width < width - 1 {
                connected_pixels.append(
                    &mut get_connected_pixels_for_pixel(genome, index + 1, width, seen_pixels)
                );
            }
        }
    }

    connected_pixels
}

#[derive(Debug, Clone)]
pub struct Individual {
    pub rgb_image: image::RgbImage,
    pub genome: Genome,

    // penalty
    pub edge_value_fitness: f64,
    pub connectivity_fitness: f64,
    pub overall_deviation_fitness: f64,
    pub euclidean_distance: Vec<Vec<Vec<Vec<f64>>>>,
}

impl Individual {
    pub fn new(image_path: &str) -> Individual {
        let rgb_image = Individual::open_image_as_rgb(image_path);
        // randomly choose the type of every field in the genome
        let mut genome = Vec::with_capacity(
            (rgb_image.width() * rgb_image.height()).try_into().unwrap()
        );
        for _ in 0..rgb_image.width() * rgb_image.height() {
            // let value = 3;
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

        Individual {
            rgb_image,
            genome,
            edge_value_fitness: 0.0,
            connectivity_fitness: 0.0,
            overall_deviation_fitness: 0.0,
            euclidean_distance: Vec::new(),
        }
    }

    pub fn open_image_as_rgb(image_path: &str) -> image::RgbImage {
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

    fn get_cluster_map(&self) -> Vec<Vec<usize>> {
        // create two-dimensional vector to store the cluster. Every pixel has a cluster id assigned

        let width: i64 = self.rgb_image.width().try_into().unwrap();
        let height: i64 = self.rgb_image.height().try_into().unwrap();
        let mut cluster_map: Vec<Vec<usize>> =
            vec![
                vec![0; width as usize];
                height  as usize
            ];

        let mut next_unused_cluster_id = 1;
        for row in 0..height as usize {
            for column in 0..width as usize {
                if cluster_map[row][column] != 0 {
                    continue;
                }
                let connected_pixels = get_connected_pixels_for_pixel(
                    &self.genome,
                    (column + row * (width as usize)) as i64,
                    width,
                    &mut vec![]
                );

                let mut cluster_id: usize = 0;
                for pixel in connected_pixels.clone().iter() {
                    let column = (pixel % width) as usize;
                    let row = (pixel / width) as usize;
                    cluster_id = cluster_map[row][column];

                    if cluster_id != 0 {
                        break;
                    }
                }

                if cluster_id == 0 {
                    cluster_id = next_unused_cluster_id;
                    next_unused_cluster_id += 1;
                }

                for pixel in connected_pixels.into_iter() {
                    let column = (pixel % width) as usize;
                    let row = (pixel / width) as usize;
                    cluster_map[row][column] = cluster_id;
                }
            }
        }

        cluster_map
    }

    pub fn get_segments_image(&self) -> RgbImage {
        let width: usize = self.rgb_image.width().try_into().unwrap();
        let height: usize = self.rgb_image.height().try_into().unwrap();

        let clustered_image = self.get_cluster_map();
        let mut image = self.rgb_image.clone();

        for row in 0..height as usize {
            for column in 0..width as usize {
                let pixel = image.get_pixel_mut(column as u32, row as u32);
                let segment = clustered_image[row][column];

                pixel.0[0] = (segment % 255) as u8;
                pixel.0[1] = (segment % 255) as u8;
                pixel.0[2] = (segment % 255) as u8;
            }
        }

        image
    }

    pub fn update_objectives(&mut self) {
        // get the phenotype for the image
        let clustered_image = self.get_cluster_map();

        let width: u32 = self.rgb_image.width().try_into().unwrap();
        let height: u32 = self.rgb_image.height().try_into().unwrap();

        // define result values for the three objectives
        let mut edge_value_fitness: f64 = 0.0;
        let mut connectivity_fitness: f64 = 0.0;
        let mut overall_deviation_fitness: f64 = 0.0;

        // Map which holds the sums for each color for all pixels in one cluster (key: cluster_id, (sum_r, sum_g, sum_b, number_of_pixels))
        let mut overall_deviation_map: HashMap<usize, (f64, f64, f64, u32)> = HashMap::new();

        // Calculate Objective: Edge Value & Connectivity
        // Bot need to loop over all pixels, get the immediate neighbors and do something if they are not in the same segment
        for row in 0..height as usize {
            for column in 0..width as usize {
                for x_offset in -1 as i32..=1 {
                    for y_offset in -1 as i32..=1 {
                        if
                            (row == 0 && y_offset == -1) ||
                            ((row as u32) == height - 1 && y_offset == 1) ||
                            (column == 0 && x_offset == -1) ||
                            ((column as u32) == width - 1 && x_offset == 1)
                        {
                            continue;
                        }
                        // check if the pixel is in the same cluster. If yes ignore it
                        if
                            clustered_image[row][column] ==
                            clustered_image[((row as i32) + y_offset) as usize]
                                [((column as i32) + x_offset) as usize]
                        {
                            continue;
                        }
                        // Edge Value := get the euclidian distance for all the neighbors which are not in the same segment
                        edge_value_fitness +=
                            self.euclidean_distance[row][column][(y_offset + 1) as usize]
                                [(x_offset + 1) as usize];

                        // Connectivity Value
                        connectivity_fitness +=
                            1.0 / (get_nearest_neighbor_value(x_offset, y_offset) as f64);
                    }
                }

                // aggregate all the pixels colors for one image to calculate the centroid later
                overall_deviation_map
                    .entry(clustered_image[row][column])
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

        // Calculate Objective: Overall Deviation

        // calculate the centroid color for each cluster by consuming the aggregated map
        let mut cluster_centroid_map: HashMap<usize, (f64, f64, f64)> = HashMap::new();
        for (cluster_id, (sum_red, sum_green, sum_blue, count)) in overall_deviation_map {
            cluster_centroid_map.insert(cluster_id, (
                sum_red / (count as f64),
                sum_green / (count as f64),
                sum_blue / (count as f64),
            ));
        }

        // formular states iterate of all pixels in all segments, which translates to loop over all pixels and get the segment for the pixel
        // for every pixel get the distance to the centroid pixel and add it to the deviation
        overall_deviation_fitness = 0.0;
        for row in 0..height as usize {
            for column in 0..width as usize {
                let current_pixel = self.rgb_image.get_pixel(column as u32, row as u32);
                let centroid_pixel = cluster_centroid_map
                    .get(&clustered_image[row][column])
                    .unwrap();
                overall_deviation_fitness += (
                    ((current_pixel[0] as f64) - centroid_pixel.0).powi(2) +
                    ((current_pixel[1] as f64) - centroid_pixel.1).powi(2) +
                    ((current_pixel[2] as f64) - centroid_pixel.2).powi(2)
                ).sqrt();
            }
        }
        self.edge_value_fitness = edge_value_fitness;
        self.connectivity_fitness = connectivity_fitness;
        self.overall_deviation_fitness = overall_deviation_fitness;
    }

    /**
     * Solution x dominates solution y, (x y), if:
        – x is better than y in at least one objective,
        – x is not worse than y in all other objectives
     */
    pub fn dominates(&self, other: &Individual) -> bool {
        let better_in_atleast_one_objective =
            self.edge_value_fitness > other.edge_value_fitness || // higher score is better
            self.connectivity_fitness < other.connectivity_fitness || // lower score is better
            self.overall_deviation_fitness < other.overall_deviation_fitness; // lower score is better

        let worse_in_all_objectives =
            self.edge_value_fitness <= other.edge_value_fitness &&
            self.connectivity_fitness >= other.connectivity_fitness &&
            self.overall_deviation_fitness >= other.overall_deviation_fitness;

        better_in_atleast_one_objective && !worse_in_all_objectives
    }
}
