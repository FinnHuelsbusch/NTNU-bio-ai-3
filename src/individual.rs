use std::{ cmp::Ordering, collections::{ BinaryHeap, HashMap, HashSet }, vec };
use image::{ ImageBuffer, Rgb, RgbImage, GrayImage };
use imageproc::edges::canny;
use rand::Rng;

use crate::{
    config::{ self, Config },
    distance::{ get_nearest_neighbor_value, EuclideanDistanceMap },
    global_data::GlobalData,
};

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

pub fn get_mst_genome(
    rgb_image: &image::RgbImage,
    distance_map: &Vec<Vec<Vec<Vec<f64>>>>
) -> Genome {
    #[derive(Debug)]
    struct MSTelement {
        row: usize,
        column: usize,
        direction: Connection,
        distance: f64,
    }

    impl PartialEq for MSTelement {
        fn eq(&self, other: &Self) -> bool {
            self.distance == other.distance
        }
    }

    impl Eq for MSTelement {}

    impl PartialOrd for MSTelement {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            self.distance.partial_cmp(&other.distance)
        }
    }

    impl Ord for MSTelement {
        fn cmp(&self, other: &Self) -> Ordering {
            self.distance.partial_cmp(&other.distance).unwrap_or(Ordering::Equal)
        }
    }

    // init the genome with none
    let mut genome =
        vec![Connection::None; (rgb_image.width() * rgb_image.height()).try_into().unwrap()];
    let mut unseen_pixels: HashSet<(usize, usize)> = HashSet::new();
    for row in 0..rgb_image.height() {
        for column in 0..rgb_image.width() {
            unseen_pixels.insert((row as usize, column as usize));
        }
    }
    let mut mst: BinaryHeap<MSTelement> = BinaryHeap::new();
    // chose a random pixel to start
    let start_row = rand::thread_rng().gen_range(0..rgb_image.height()) as usize;
    let start_column = rand::thread_rng().gen_range(0..rgb_image.width()) as usize;
    unseen_pixels.remove(&(start_row, start_column));
    // add the start pixel to the mst
    if start_row > 0 {
        mst.push(MSTelement {
            row: start_row - 1,
            column: start_column,
            direction: Connection::Down,
            distance: distance_map[start_row][start_column][0][1],
        });
    }
    if start_row < (rgb_image.height() as usize) - 1 {
        mst.push(MSTelement {
            row: start_row + 1,
            column: start_column,
            direction: Connection::Up,
            distance: distance_map[start_row][start_column][2][1],
        });
    }
    if start_column > 0 {
        mst.push(MSTelement {
            row: start_row,
            column: start_column - 1,
            direction: Connection::Right,
            distance: distance_map[start_row][start_column][1][0],
        });
    }
    if start_column < (rgb_image.width() as usize) - 1 {
        mst.push(MSTelement {
            row: start_row,
            column: start_column + 1,
            direction: Connection::Left,
            distance: distance_map[start_row][start_column][1][2],
        });
    }

    while !unseen_pixels.is_empty() {
        let mst_element = mst.pop().unwrap();
        if unseen_pixels.remove(&(mst_element.row, mst_element.column)) {
            genome[mst_element.row * (rgb_image.width() as usize) + mst_element.column] =
                mst_element.direction;
            // Add the neighbors of the pixel to the mst
            for (row_adjustment, column_adjustment, dir) in &[
                (0, -1, Connection::Right),
                (0, 1, Connection::Left),
                (-1, 0, Connection::Down),
                (1, 0, Connection::Up),
            ] {
                let new_row = ((mst_element.row as isize) + row_adjustment) as usize;
                let new_col = ((mst_element.column as isize) + column_adjustment) as usize;
                if
                    new_row < (rgb_image.height() as usize) &&
                    new_col < (rgb_image.width() as usize) &&
                    unseen_pixels.contains(&(new_row, new_col))
                {
                    mst.push(MSTelement {
                        row: new_row,
                        column: new_col,
                        direction: *dir,
                        distance: distance_map[mst_element.row][mst_element.column]
                            [(1 - row_adjustment) as usize][(1 - column_adjustment) as usize],
                    });
                }
            }
        }
    }
    genome
}

fn get_connected_pixels_for_pixel(
    genome: &Genome,
    index: i64,
    width: i64,
    seen_pixels: &mut Vec<i64>,
    cluster_map: &Vec<Vec<usize>>
) -> Vec<i64> {
    if seen_pixels.contains(&index) {
        return vec![];
    }

    seen_pixels.push(index);
    let mut connected_pixels = vec![index];

    let column = (index % width) as usize;
    let row = (index / width) as usize;

    // if the current pixel already has a cluster, all following pixels will also have a cluster
    if cluster_map[row][column] != 0 {
        return connected_pixels;
    }

    // check direction going to and follow the path
    let direction = genome[index as usize];
    match direction {
        Connection::None => {}
        Connection::Up => {
            // if the direction is up the index needs to be at least in the second row. So width should be at least one time in the index
            if index - width > 0 {
                connected_pixels.append(
                    &mut get_connected_pixels_for_pixel(
                        genome,
                        index - width,
                        width,
                        seen_pixels,
                        cluster_map
                    )
                );
            }
        }
        Connection::Down => {
            // if the direction is down the index needs to be at least in the second to last row. So index + width should not be higher than genome length
            if index + width < (genome.len() as i64) {
                connected_pixels.append(
                    &mut get_connected_pixels_for_pixel(
                        genome,
                        index + width,
                        width,
                        seen_pixels,
                        cluster_map
                    )
                );
            }
        }
        Connection::Left => {
            // if the direction is left the index needs bigger than the wrapping of the width. So index % width should give the index in a row and this needs to be bigger than 0
            if index % width > 0 {
                connected_pixels.append(
                    &mut get_connected_pixels_for_pixel(
                        genome,
                        index - 1,
                        width,
                        seen_pixels,
                        cluster_map
                    )
                );
            }
        }
        Connection::Right => {
            // if the direction is right the index needs to be less than the wrapping of the width. So index % width should give the index in a row and this needs to be less than width - 1 (width is the edge)
            if index % width < width - 1 {
                connected_pixels.append(
                    &mut get_connected_pixels_for_pixel(
                        genome,
                        index + 1,
                        width,
                        seen_pixels,
                        cluster_map
                    )
                );
            }
        }
    }

    connected_pixels
}

fn is_border_pixel(
    segmentation_map: &Vec<Vec<usize>>,
    row: usize,
    col: usize,
    rows: usize,
    cols: usize,
    current_segment: usize
) -> bool {
    // Check surrounding pixels for different segments
    let up = if row == 0 { None } else { Some(segmentation_map[row - 1][col]) };
    let down = if row == rows - 1 { None } else { Some(segmentation_map[row + 1][col]) };
    let left = if col == 0 { None } else { Some(segmentation_map[row][col - 1]) };
    let right = if col == cols - 1 { None } else { Some(segmentation_map[row][col + 1]) };

    // Check if any surrounding pixel has a different segment
    if let Some(segment) = up {
        if segment != current_segment {
            return true;
        }
    }
    if let Some(segment) = down {
        if segment != current_segment {
            return true;
        }
    }
    if let Some(segment) = left {
        if segment != current_segment {
            return true;
        }
    }
    if let Some(segment) = right {
        if segment != current_segment {
            return true;
        }
    }

    false
}
#[derive(Debug, Clone)]
pub struct Individual {
    pub genome: Genome,
    needs_update: bool,

    

    // penalty
    edge_value_fitness: f64,
    connectivity_fitness: f64,
    overall_deviation_fitness: f64,
    fitness: f64,
}

impl Individual {
    pub fn new_random(global_data: &GlobalData) -> Individual {
        let genome: Genome;
        genome = Individual::init_random_genome(global_data.rgb_image);
        Individual {
            genome,
            needs_update: true,
            fitness: 0.0, // higher is better
            edge_value_fitness: 0.0,
            connectivity_fitness: 0.0,
            overall_deviation_fitness: 0.0,
        }
    }

    pub fn new_with_genome(genome: &Genome) -> Individual {
        Individual {
            genome: genome.clone(),
            needs_update: true,
            fitness: 0.0,
            edge_value_fitness: 0.0,
            connectivity_fitness: 0.0,
            overall_deviation_fitness: 0.0,
        }
    }

    pub fn needs_update(&self) -> bool {
        self.needs_update
    }

    pub fn set_needs_update(&mut self) {
        self.needs_update = true;
    }

    pub fn open_image_as_rgb(image_path: &str) -> RgbImage {
        let img = image::open(image_path).unwrap();
        img.to_rgb8()
    }

    pub fn open_image_as_edge_map(image_path: &str, low: f32, high: f32) -> GrayImage {
        let img = image::open(image_path).unwrap();

        canny(&img.to_luma8(), low, high)
    }

    fn init_random_genome(rgb_image: &image::RgbImage) -> Genome {
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
        genome
    }

    pub fn get_cluster_map(&self, width: i64, height: i64) -> Vec<Vec<usize>> {
        // create two-dimensional vector to store the cluster. Every pixel has a cluster id assigned

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
                    &mut vec![],
                    &cluster_map
                );

                let last_pixel = connected_pixels.last().unwrap();
                let column = (last_pixel % width) as usize;
                let row = (last_pixel / width) as usize;
                let mut cluster_id: usize = cluster_map[row][column];

                if cluster_id == 0 {
                    cluster_id = next_unused_cluster_id;
                    next_unused_cluster_id += 1;
                }

                for pixel in connected_pixels.into_iter() {
                    // TODO evaluate if it is faster to check if the pixel is already assigned to a cluster and break as soon as the first pixel is found.
                    let column = (pixel % width) as usize;
                    let row = (pixel / width) as usize;
                    cluster_map[row][column] = cluster_id;
                }
            }
        }

        cluster_map
    }

    pub fn get_segments_image(&self, global_data: &GlobalData) -> RgbImage {
        let clustered_image = self.get_cluster_map(
            global_data.width as i64,
            global_data.height as i64
        );
        let mut image = ImageBuffer::from_pixel(
            global_data.width as u32,
            global_data.height as u32,
            Rgb([0u8, 0u8, 0u8])
        );

        let colorpalett = vec![
            (25, 200, 56),
            (25, 200, 56),
            (138, 42, 226),
            (158, 72, 0),
            (241, 76, 192),
            (163, 163, 163),
            (255, 195, 0),
            (0, 214, 255),
            (1, 62, 255),
            (255, 123, 0)
        ];

        for row in 0..global_data.height {
            for column in 0..global_data.width {
                let pixel = image.get_pixel_mut(column as u32, row as u32);
                let segment = clustered_image[row][column];
                let color = colorpalett[segment % colorpalett.len()];
                *pixel = image::Rgb([color.0, color.1, color.2]);
            }
        }

        image
    }

    pub fn update_objectives(&mut self, config: &Config, global_data: &GlobalData) {
        // get the phenotype for the image
        let clustered_image = self.get_cluster_map(
            global_data.width as i64,
            global_data.height as i64
        );

        // define result values for the three objectives
        let mut edge_value_fitness: f64 = 0.0;
        let mut connectivity_fitness: f64 = 0.0;
        let mut overall_deviation_fitness: f64 = 0.0;

        let rgb_image: &RgbImage = global_data.rgb_image;
        let euclidean_distance_map: &EuclideanDistanceMap = global_data.euclidean_distance_map;

        // Map which holds the sums for each color for all pixels in one cluster (key: cluster_id, (sum_r, sum_g, sum_b, number_of_pixels))
        let mut overall_deviation_map: HashMap<usize, (f64, f64, f64, u32)> = HashMap::new();

        // Calculate Objective: Edge Value & Connectivity
        // Bot need to loop over all pixels, get the immediate neighbors and do something if they are not in the same segment
        for row in 0..global_data.height {
            for column in 0..global_data.width {
                for x_offset in -1 as i32..=1 {
                    for y_offset in -1 as i32..=1 {
                        if
                            (row == 0 && y_offset == -1) ||
                            (row == global_data.height - 1 && y_offset == 1) ||
                            (column == 0 && x_offset == -1) ||
                            (column == global_data.width - 1 && x_offset == 1)
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
                            euclidean_distance_map[row][column][(y_offset + 1) as usize]
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
                        *sum_red += rgb_image.get_pixel(column as u32, row as u32)[0] as f64;
                        *sum_green += rgb_image.get_pixel(column as u32, row as u32)[1] as f64;
                        *sum_blue += rgb_image.get_pixel(column as u32, row as u32)[2] as f64;
                        *count += 1;
                    })
                    .or_insert((
                        rgb_image.get_pixel(column as u32, row as u32)[0] as f64,
                        rgb_image.get_pixel(column as u32, row as u32)[1] as f64,
                        rgb_image.get_pixel(column as u32, row as u32)[2] as f64,
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

        for row in 0..global_data.height as usize {
            for column in 0..global_data.width as usize {
                let current_pixel = rgb_image.get_pixel(column as u32, row as u32);
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
        self.fitness =
            edge_value_fitness * config.edge_value_multiplier +
            -connectivity_fitness * config.connectivity_multiplier +
            -overall_deviation_fitness * config.overall_deviation_multiplier;
        self.needs_update = false;
    }

    pub fn get_objectives(&self) -> (f64, f64, f64) {
        if self.needs_update {
            panic!("Objectives need to be updated before getting them");
        }
        (self.edge_value_fitness, self.connectivity_fitness, self.overall_deviation_fitness)
    }

    pub fn get_fitness(&self) -> f64 {
        if self.needs_update {
            panic!("Fitness needs to be updated before getting it");
        }
        self.fitness
    }

    /**
     * Solution x dominates solution y, (x y), if:
        – x is better than y in at least one objective,
        – x is not worse than y in all other objectives
     */
    pub fn dominates(&self, other: &Individual) -> bool {
        if self.needs_update || other.needs_update {
            panic!("Objectives need to be updated before comparing them");
        }
        let better_in_atleast_one_objective =
            self.edge_value_fitness > other.edge_value_fitness || // higher score is better
            self.connectivity_fitness < other.connectivity_fitness || // lower score is better
            self.overall_deviation_fitness < other.overall_deviation_fitness; // lower score is better

        let worse_in_any_objective =
            self.edge_value_fitness < other.edge_value_fitness ||
            self.connectivity_fitness > other.connectivity_fitness ||
            self.overall_deviation_fitness > other.overall_deviation_fitness;

        better_in_atleast_one_objective && !worse_in_any_objective
    }

    pub fn get_segment_border_image(&self, global_data: &GlobalData) -> RgbImage {
        let mut image = ImageBuffer::from_pixel(
            global_data.width as u32,
            global_data.height as u32,
            Rgb([0u8, 0u8, 0u8])
        );
        let border_image = self.get_border_map(global_data);

        for row in 0..global_data.height {
            for column in 0..global_data.width {
                let pixel = image.get_pixel_mut(column as u32, row as u32);
                let color = border_image[row][column];
                *pixel = image::Rgb([color, color, color]);
            }
        }

        image
    }

    pub fn get_segment_border_image_inline(&self, global_data: &GlobalData) -> RgbImage {
        let mut image = global_data.rgb_image.clone();
        let border_image = self.get_border_map(global_data);

        for row in 0..global_data.height {
            for column in 0..global_data.width {
                let pixel = image.get_pixel_mut(column as u32, row as u32);
                let color = border_image[row][column];
                if color == 0 {
                    *pixel = image::Rgb([0, 255 - color, 0]);
                }
            }
        }

        image
    }

    pub fn get_border_map(&self, global_data: &GlobalData) -> Vec<Vec<u8>> {
        let segmentation_map = self.get_cluster_map(
            global_data.width as i64,
            global_data.height as i64
        );
        let rows = segmentation_map.len();
        let cols = segmentation_map[0].len();

        let mut border_map: Vec<Vec<u8>> = vec![vec![0; cols]; rows];

        // Iterate over each pixel
        for i in 0..rows {
            for j in 0..cols {
                let current_segment = segmentation_map[i][j];

                // Check if current pixel is on the border
                let is_border = is_border_pixel(
                    &segmentation_map,
                    i,
                    j,
                    rows,
                    cols,
                    current_segment
                );
                if is_border {
                    border_map[i][j] = 0;
                } else {
                    border_map[i][j] = 255;
                }
            }
        }

        // Fill the border of the image with 0s
        for i in 0..rows {
            border_map[i][0] = 0;
            border_map[i][cols - 1] = 0;
        }
        for j in 0..cols {
            border_map[0][j] = 0;
            border_map[rows - 1][j] = 0;
        }

        border_map
    }
}
