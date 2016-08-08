#![feature(alloc_system)]
extern crate alloc_system;
extern crate katome;
extern crate toml;
extern crate rustc_serialize;
extern crate log4rs;
// extern crate flame;
use katome::asm::assembler::assemble;
use toml::{Parser, Value};
use std::fs::File;
use std::io::Read;



fn main() {
    log4rs::init_file("./config/log4rs.yaml", Default::default()).unwrap();
    let config = parse_config("./config/settings.toml".to_string());
    println!("{:?}", config);
    assemble(config.input_path,
             config.output_path,
             config.original_genome_length,
             config.minimal_weight_threshold);
    // flame::dump_html(&mut File::create("flame-graph.html").unwrap()).unwrap();
}

#[derive(Debug)]
#[derive(RustcDecodable)]
pub struct GenomeConfig {
    input_path: String,
    output_path: String,
    original_genome_length: usize,
    minimal_weight_threshold: usize,
}

/// Attempt to load and parse the config file into our Config struct.
/// If a file cannot be found, return a default Config.
/// If we find a file but cannot parse it, panic
pub fn parse_config(path: String) -> GenomeConfig {
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
            println!("{}:{}:{}-{}:{} error: {}",
                     path,
                     loline,
                     locol,
                     hiline,
                     hicol,
                     err.desc);
        }
        panic!("Exiting!");
    }

    let config = Value::Table(toml.unwrap());
    match toml::decode(config) {
        Some(t) => t,
        None => panic!("Error while deserializing config"),
    }
}
