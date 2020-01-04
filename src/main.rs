#[macro_use]
extern crate crossbeam_channel;
#[macro_use]
extern crate clap;

mod configuration;
mod industruino;
mod messages;
mod temperature;

use crate::temperature::Temperature;
use clap::{App, Arg};
use configuration::parse_config;
use industruino::Industruino;

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let config = parse_config(matches).expect("Parsing failed...");

    let mut industruino = Industruino::new(config.name.clone(), config.secret_token.clone());

    let temperatures: Vec<Temperature> = config
        .temperatures
        .iter()
        .map(|temperature_libelle| {
            industruino
                .create_attached_temperature_sensor(temperature_libelle.clone().as_str(), 21.5)
        })
        .collect();

    for temperature in temperatures {
        temperature.run()
    }

    industruino.connect(&config.hostname, config.port).run();
}
