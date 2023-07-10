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

use std::path::PathBuf;
use log::{error, info};

use crate::hdf_file::HDFWriter;
use crate::merger::Merger;
use crate::event_builder::EventBuilder;
use crate::pad_map::PadMap;
use crate::config::Config;

fn flush_final_event(mut evb: EventBuilder, writer: HDFWriter) {
    if let Some(event) = evb.flush_final_event() {
        match writer.write_event(event) {
            Ok(()) => (),
            Err(e) => {
                log::error!("HDFWriter recieved an error: {}", e);
                return;
            }
        }
    }
}

fn process_run(mut merger: Merger, mut evb: EventBuilder, writer: HDFWriter) {
        let mut count =0;
        let mut flush_count = 0;
        let flush_val = 100;
        loop {
            let frame = match merger.get_next_frame() {
                Ok(f) => f,
                Err(crate::error::MergerError::EndOfMerge) => {
                    flush_final_event(evb, writer);
                    break;
                }
                Err(e) => {
                    log::error!("Merger recieved an error: {}", e);
                    return;
                }
            };

            let maybe_event = match evb.append_frame(frame) {
                Ok(option) => match option {
                    Some(event) => event,
                    None => continue
                }
                Err(e) => {
                    log::error!("Event builder recieved an error: {}", e);
                    return;
                }
            };

            match writer.write_event(maybe_event) {
                Ok(()) => {
                    count += 1;
                    if count == flush_val {
                        count = 0;
                        flush_count += 1;
                        log::info!("{} events processed", flush_count *flush_val);
                    }
                },
                Err(e) => {
                    log::error!("HDFWriter recieved an error: {}", e);
                    return;
                }
            }
        }


}

#[allow(unreachable_code, dead_code)]
fn main() {
    //Setup logging
    simplelog::TermLogger::init(simplelog::LevelFilter::Info, 
        simplelog::Config::default(),
        simplelog::TerminalMode::Mixed, 
        simplelog::ColorChoice::Auto)
    .unwrap();
    
    let clocker = std::time::Instant::now();
    info!("Starting up rusted graw...\n");

    info!("Reading configuration...\n");
    //TEMP -- This is the basic configuration
    let config_path = PathBuf::from("temp.yaml");
    let config = match Config::read_config_file(&config_path) {
        Ok(c) => c,
        Err(e) => {
            error!("Error reading configuration: {} Shutting down.\n", e);
            return;
        }
    };
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


    info!("Configuration parsed.\n GRAW run path: {}\n HDF file path: {}\n Pad map file path: {}\n", run_path.display(), hdf_path.display(), config.pad_map_path.display());
    info!("Initializing resources...\n");
    //Setup resources
    let pad_map = match PadMap::new(&config.pad_map_path) {
        Ok(pm) => pm,
        Err(e) => {
            error!("PadMap error at creation: {} Shutting down.\n", e);
            return;
        }
    };

    //Initialize the merger, event builder, and hdf writer
    let merger = match  Merger::new(&run_path) {
        Ok(m) => m,
        Err(e) => {
            error!("An error was encountered initializing the merger: {} Shutting down.\n", e);
            return;
        }
    };
    let evb = EventBuilder::new(pad_map);
    let writer = match HDFWriter::new(&hdf_path) {
        Ok(hdf) => hdf,
        Err(e) => {
            error!("An error was encountered initializing the hdf file: {} Shutting down.\n", e);
            return;
        }
    };
    info!("Merger ready. Running...\n");

    process_run(merger, evb, writer);

    info!("Done.\n");

    info!("Time ellapsed: {:?}", clocker.elapsed());
    return;

}
