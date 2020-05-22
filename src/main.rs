use std::env;
use std::process;

use necromancer::Config;

fn main() {
    let args: Vec<String> = env::args().collect();

    let config = Config::new(&args).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    println!("Executing file {}", config.filename);

    if let Err(e) = necromancer::run(config) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }
}

