use rand::Rng;

use crate::{config::Config, individual::{Connection, Genome}, population::Population};



fn flip_one_bit(genome: &mut Genome) {
    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..genome.len());
    let new_connection = match rand::thread_rng().gen_range(0..5) {
        0 => Connection::None,
        1 => Connection::Up,
        2 => Connection::Down,
        3 => Connection::Left,
        4 => Connection::Right,
        _ => panic!("Invalid connection value"),
    };
    genome[index] = new_connection;
}

pub fn mutate(
    population: &mut Population,
    config: &Config,
) -> Population {
    let mut rng = rand::thread_rng();
    let mut children: Population = population.clone();
    for mutation_config in config.mutations.iter() {
        // Calculate the number of crossovers which should happen for the specific config
        let number_of_mutations: u64 = ((config.population_size as f64)
            * mutation_config.probability.unwrap_or(0.0))
        .ceil() as u64;

        for _ in 0..number_of_mutations {
            let individual_index: usize = rng.gen_range(0..config.population_size);

            let child_genome: Genome = match mutation_config.name.as_str() {
                "flip_one_bit" => {
                    let mut child_genome = children[individual_index].genome.clone();
                    flip_one_bit(&mut child_genome);
                    child_genome
                }
                _ => panic!(
                    "Didn't have an Implementation for mutation function: {:?}",
                    mutation_config.name.as_str()
                ),
            };

            let mut child = children[individual_index].clone();
            child.genome = child_genome;
            child.update_objectives();
            children.push(child);
            
        }
    }

    return children;
}
