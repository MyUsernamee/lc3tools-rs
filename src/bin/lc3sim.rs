use clap::Parser;
use lc3tools_rs::{Debugger, LC3Simulator};
use log::warn;
use simple_logger::SimpleLogger;
use std::{fs::read, path::PathBuf};

#[derive(Parser, Debug)]
pub struct Cli {
    /// Obj files to load. Program counter will be set to the
    /// orig of the last obj file.
    obj_files: Vec<PathBuf>,
    /// Don't open repl even on execeptions.
    #[arg(long)]
    no_repl: bool,
    /// Unless no-repl is provided, 
    /// add a breakpoint at the given address, open a repl when hit,
    /// otherwise does nothing.
    #[arg(short, long)]
    breakpoint: Vec<u16>, 
    #[arg(long)]
    verbose: bool
}

fn main() {
    let cli = Cli::parse();
    let mut sim = LC3Simulator::with_os();

    if cli.verbose {
        SimpleLogger::new().init().unwrap();
        log::set_max_level(log::LevelFilter::Trace);
    }

    if cli.obj_files.len() == 0 {
        warn!("No obj files provided.");
    }

    for file in cli.obj_files {
        let file_data = { read(file).expect("Unable to load file.") };
        sim.load_obj(file_data, true)
            .expect("Unable to read obj file.");
    }

    if cli.no_repl {
        run_no_repl(sim);
        return;
    }

    let mut debugger = Debugger::default();
    debugger.run().expect("Error rendering debugger.");

}

fn run_no_repl(mut sim: LC3Simulator) {
    let out_callback = |sim: &mut LC3Simulator, v: u16| {
        print!("{}", String::from_utf8([v as u8].to_vec()).unwrap());
        sim.write(1u16<<15, 0xFE04); // Ready for next character.
    };

    sim.write(1u16<<15, 0xFE04); // Ready for next character.
    sim.add_write_callback(0xFE06, out_callback);

    while sim.step() {
    }
}
