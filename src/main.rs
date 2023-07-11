mod constants;
mod error;
mod event;
mod graw_frame;
mod graw_file;
mod merger;
mod pad_map;
mod event_builder;
mod hdf_file;
mod config;
mod asad_stack;
mod process;

use std::path::PathBuf;
use log::{error, info};

use crate::process::process_run;
use crate::hdf_file::HDFWriter;
use crate::merger::Merger;
use crate::event_builder::EventBuilder;
use crate::pad_map::PadMap;
use crate::config::Config;

fn print_help_string() {
    println!("----- rusted_graw -------");
    println!("To run use the command below");
    println!("cargo run --release <my_config.yaml>");
    println!("Replace the <my_config.yaml> with the path to a configuration file.");
    println!("See the README for more details on rusted_graw.");
}

#[allow(unreachable_code, dead_code)]
fn main() {

    let args: Vec<String> = std::env::args().collect();
    if args.len() <= 1 {
        print_help_string();
        return;
    } else if args[1] == "--help" {
        print_help_string();
        return;
    }

    let config_filestr = &args[1];
    let config_path = PathBuf::from(config_filestr);
    let config = match Config::read_config_file(&config_path) {
        Ok(c) => c,
        Err(e) => {
            println!("Error reading configuration: {} Shutting down.\n", e);
            return;
        }
    };

    let log_file_path = config.get_log_file_name().unwrap();
    simplelog::WriteLogger::init(simplelog::LevelFilter::Info, 
                                simplelog::Config::default(),
                        std::fs::File::create(log_file_path.clone()).unwrap())
                            .unwrap();
    
    let run_path = match config.get_run_directory() {
        Ok(p) => p,
        Err(e) => {
            error!("Config error recieved: {}", e);
            return;
        }
    };
    let hdf_path = match config.get_hdf_file_name() {
        Ok(p) => p,
        Err(e) => {
            error!("Config error recieved: {}", e);
            return;
        }
    };

    let clocker = std::time::Instant::now();

    println!("----- rusted_graw -------");
    println!("GRAW run directory: {}", run_path.display());
    println!("HDF5 Output File: {}", hdf_path.display());
    println!("Pad map File: {}", config.pad_map_path.display());
    println!("Log file: {}", log_file_path.display());

    info!("Starting up rusted graw...\n");
    info!("Configuration parsed.\n GRAW run path: {}\n HDF file path: {}\n Pad map file path: {}\n", run_path.display(), hdf_path.display(), config.pad_map_path.display());
    info!("Initializing resources...\n");

    //Setup resources
    let pad_map = match PadMap::new(&config.pad_map_path) {
        Ok(pm) => pm,
        Err(e) => {
            println!("Error at PadMap! Check log file for details.");
            error!("PadMap error at creation: {} Shutting down.\n", e);
            return;
        }
    };

    //Initialize the merger, event builder, and hdf writer
    let merger = match  Merger::new(&run_path) {
        Ok(m) => m,
        Err(e) => {
            println!("Error at Merger! Check log file for details.");
            error!("An error was encountered initializing the merger: {} Shutting down.\n", e);
            return;
        }
    };
    let evb = EventBuilder::new(pad_map);
    let writer = match HDFWriter::new(&hdf_path) {
        Ok(hdf) => hdf,
        Err(e) => {
            println!("Error at HDFWriter! Check log file for details.");
            error!("An error was encountered initializing the hdf file: {} Shutting down.\n", e);
            return;
        }
    };
    info!("Merger ready. Running...\n");

    match process_run(merger, evb, writer) {
        Ok(_) => (),
        Err(e) => {
            println!("An error occurred while processing! See the log for more info!");
            error!("Error while processing: {}", e);
        }
    }

    info!("Done.\n");

    info!("Time ellapsed: {:?}", clocker.elapsed());
    return;

}
