mod commands;
mod models;
mod schema;

#[macro_use]
extern crate log;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate clap;
extern crate dotenv;
extern crate env_logger;
extern crate reqwest;
extern crate serde_json;
extern crate crossbeam_utils;

fn main() {
    dotenv::dotenv().unwrap();

    env_logger::init();

    let yaml = load_yaml!("cli.yml");

    let matches = clap::App::from_yaml(yaml).get_matches();

    match matches.subcommand() {
        ("emoticons", matches) => commands::emoticons::run(matches),
        _ => unimplemented!(),
    }
}
