use std::{fs::read, path::PathBuf};
use clap::Parser;
use lc3tools_rs::LC3Simulator;

#[derive(Parser, Debug)]
pub struct Cli {
    obj_file: PathBuf
}

fn main() {
    let cli = Cli::parse();
    
    let file_data = {
        read(cli.obj_file).expect("Unable to load file.")
    };

    let mut sim = LC3Simulator::new();
    sim.load_obj(file_data, true).expect("Unable to read obj file.");

    println!("{:?}", sim.get_annotation()[0x3000..0x30F0].to_vec());
}
