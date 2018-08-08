mod commands;
mod error;
mod models;
mod schema;

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate clap;
extern crate crossbeam_utils;
extern crate dotenv;
extern crate env_logger;
extern crate hyper;
extern crate reqwest;
extern crate serde_json;

use clap::App;

fn init() {
    if let Err(e) = dotenv::dotenv() {
        warn!("dotenv: {}", e);
    }

    env_logger::init();
}

fn main() {
    init();

    let yaml = load_yaml!("cli.yml");

    let result = match App::from(yaml).get_matches().subcommand() {
        ("emoticons", Some(matches)) => commands::emoticons::run(matches),
        ("server", Some(matches)) => commands::server::run(matches),
        _ => unimplemented!(),
    };

    match result {
        Ok(_) => info!("success"),
        Err(e) => error!("{}", e),
    }
}
