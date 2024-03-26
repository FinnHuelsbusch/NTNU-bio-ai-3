use crate::{config::Config, individual::Individual};


pub type Population = Vec<Individual>;

pub fn initialize_random_population(config: &Config) -> Population {
    let mut population = Vec::new();
    for _ in 0..config.population_size {
        let mut individual = Individual::new(&config.picture_path);
        individual.update_objectives();
        population.push(individual);
    }
    population
}

pub fn non_dominated_sort(population: &Population) -> Vec<Vec<usize>> {
    let mut frontiers = vec![Vec::new()];
    let mut dominated_by = vec![Vec::new(); population.len()];
    let mut dominates = vec![0; population.len()];
    let mut rank = vec![0; population.len()];

    // Calculate the number of solutions that dominate each solution
    for i in 0..population.len() {
        for j in 0..population.len() {
            if i == j {
                continue;
            } else if population[i].dominates(&population[j]) {
                dominates[i] += 1;
            } else if population[j].dominates(&population[i]) {
                dominated_by[i].push(j);
            }
        }
        // If the solution is not dominated by any other solution, it is a member of the first frontier
        if dominates[i] == 0 {
            rank[i] = 0;
            frontiers[0].push(i);
        }
    }

    let mut i = 0;
    // Calculate the rest of the frontiers
    // while current frontier is not empty
    while !frontiers[i].is_empty() {
        // Calculate the next frontier
        let mut next_front = Vec::new();
        for &p in &frontiers[i] {
            for &q in &dominated_by[p] {
                dominates[q] -= 1;
                if dominates[q] == 0 {
                    rank[q] = i + 1;
                    next_front.push(q);
                }
            }
        }
        i += 1;
        frontiers.push(next_front);
    }

    frontiers.pop();
    frontiers
}