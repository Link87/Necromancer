use std::env;
use std::process;

use log::{error, info};
use necromancer::Config;

fn main() {
    let args: Vec<String> = env::args().collect();

    env_logger::init();

    let config = Config::new(&args).unwrap_or_else(|err| {
        error!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    info!("Executing file {}", config.filename);

    if let Err(e) = necromancer::run(config) {
        error!("Application error: {}", e);
        process::exit(1);
    }
}
