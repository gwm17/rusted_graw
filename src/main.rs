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
use indicatif::{ProgressBar, ProgressStyle};

use crate::hdf_file::HDFWriter;
use crate::merger::Merger;
use crate::event_builder::EventBuilder;
use crate::pad_map::PadMap;
use crate::config::Config;
use crate::constants::SIZE_UNIT;

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
        let progress = ProgressBar::new(*merger.get_total_data_size());
        let style = ProgressStyle::with_template("[{elapsed}] {bar:40.cyan/blue} {bytes}/{total_bytes} {msg}").unwrap();
        progress.set_style(style);
        let total_data_size = merger.get_total_data_size();
        let flush_frac = 0.01;
        let mut count =0;
        let flush_val = (*total_data_size as f64 * flush_frac) as u64;
        loop {
            let frame = match merger.get_next_frame() {
                Ok(f) => f,
                Err(crate::error::MergerError::EndOfMerge) => {
                    flush_final_event(evb, writer);
                    break;
                }
                Err(e) => {
                    println!("Merger error! Check log file for details.");
                    log::error!("Merger recieved an error: {}", e);
                    return;
                }
            };

            //bleh
            count += (frame.header.header_size as u32 * SIZE_UNIT + frame.header.n_items * frame.header.item_size as u32) as u64;
            if count > flush_val {
                progress.inc(count);
                count = 0;
            }

            let maybe_event = match evb.append_frame(frame) {
                Ok(Some(event)) => event,
                Ok(None) => {
                    continue;
                }
                Err(e) => {
                    println!("Event builder error! Check log file for details.");
                    log::error!("Event builder recieved an error: {}", e);
                    return;
                }
            };

            match writer.write_event(maybe_event) {
                Ok(()) => (),
                Err(e) => {
                    println!("Writer error! Check log file for details.");
                    log::error!("HDFWriter recieved an error: {}", e);
                    return;
                }
            }
        }

        progress.finish();


}

#[allow(unreachable_code, dead_code)]
fn main() {
    //TEMP -- This is the basic configuration
    let config_path = PathBuf::from("temp.yaml");
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

    process_run(merger, evb, writer);

    info!("Done.\n");

    info!("Time ellapsed: {:?}", clocker.elapsed());
    return;

}
