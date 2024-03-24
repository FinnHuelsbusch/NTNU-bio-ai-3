use individual::Individual;


mod individual;


fn main() {
    let filepath = "./Project 3 training_images/86016/Test image.jpg"; 
    let mut individual = Individual::new(filepath);
    individual.update_objectives();
    println!("Individual: {:?}", individual);
}
