//! Example of genome assembler using `katome` library.

extern crate katome;
extern crate toml;
extern crate rustc_serialize;
extern crate log4rs;

use katome::{Assemble, BasicAsm, Config};
use katome::collections::{PtGraph, HsGIR};
use std::fs::File;
use std::io::Read;
use toml::{Parser, Value};

fn main() {
    log4rs::init_file("./config/log4rs.yaml", Default::default()).unwrap();
    let config = parse_config("./config/settings.toml".to_string());
    println!("{:?}", config);
    // BasicAsm::assemble::<String, PtGraph>(config);
    BasicAsm::assemble_with_gir::<String, PtGraph, HsGIR>(config);
}

/// Attempt to load and parse the config file into our Config struct.
/// If a file cannot be found, return a default Config.
/// If we find a file but cannot parse it, panic
pub fn parse_config(path: String) -> Config<String> {
    let mut config_toml = String::new();
    let mut file = match File::open(&path) {
        Ok(file) => file,
        Err(_) => {
            panic!("Could not find config file!");
        }
    };

    file.read_to_string(&mut config_toml)
        .unwrap_or_else(|err| panic!("Error while reading config: [{}]", err));

    let mut parser = Parser::new(&config_toml);
    let toml = parser.parse();

    if toml.is_none() {
        for err in &parser.errors {
            let (loline, locol) = parser.to_linecol(err.lo);
            let (hiline, hicol) = parser.to_linecol(err.hi);
            println!("{}:{}:{}-{}:{} error: {}", path, loline, locol, hiline, hicol, err.desc);
        }
        panic!("Exiting!");
    }

    let config = Value::Table(toml.unwrap());
    match toml::decode(config) {
        Some(t) => t,
        None => panic!("Error while deserializing config"),
    }
}
