mod constants;
mod error;
mod event;
mod graw_frame;
mod graw_file;
mod merger;
mod pad_map;
mod event_builder;
mod hdf_file;

use std::sync::mpsc::channel;
use std::path::PathBuf;
use log::{error, info};


use crate::event::Event;
use crate::graw_frame::GrawFrame;
use crate::hdf_file::HDFWriter;
use crate::merger::Merger;
use crate::event_builder::EventBuilder;
use crate::pad_map::PadMap;

fn main() {
    //Setup logging
    simplelog::TermLogger::init(simplelog::LevelFilter::Info, 
        simplelog::Config::default(),
        simplelog::TerminalMode::Mixed, 
        simplelog::ColorChoice::Auto)
    .unwrap();

    info!("Starting up rusted graw...\n");

    //TEMP -- This is the basic configuration
    let pad_map_path: PathBuf = PathBuf::from("path");
    let graw_path: PathBuf = PathBuf::from("graw");
    let hdf_path: PathBuf = PathBuf::from("hdf");

    //Setup resources
    let (event_tx, event_rx) = channel::<Event>();
    let (frame_tx, frame_rx) = channel::<GrawFrame>();
    let pad_map = match PadMap::new(&pad_map_path) {
        Ok(pm) => pm,
        Err(e) => {
            error!("PadMap error at creation: {} Shutting down.\n", e);
            return;
        }
    };

    //Initialize the merger, event builder, and hdf writer
    let mut merger = match  Merger::new(&graw_path, frame_tx) {
        Ok(m) => m,
        Err(e) => {
            error!("An error was encountered initializing the merger: {} Shutting down.\n", e);
            return;
        }
    };
    let mut evb = EventBuilder::new(frame_rx, event_tx, pad_map);
    let hdf_writer = match HDFWriter::new(&hdf_path, event_rx) {
        Ok(hdf) => hdf,
        Err(e) => {
            error!("An error was encountered initializing the hdf file: {} Shutting down.\n", e);
            return;
        }
    };

    //Spawn event builder thread
    let evb_handle = std::thread::spawn(move || {
        match evb.run() {
            Ok(_) => info!("EventBuilder successfully completed.\n"),
            Err(e) => error!("EventBuilder ran into an error: {} Shutting down.\n", e)
        }
    });

    //Spawn hdf write thread
    let hdf_handle = std::thread::spawn(move || {
        match hdf_writer.run() {
            Ok(_) => info!("HDF writer successfully completed.\n"),
            Err(e) => error!("HDFWriter ran into an error: {} Shutting down.\n", e)
        }
    });

    //Run the merger in the main thread
    match merger.run() {
        Ok(_) => info!("Merger successfully completed.\n"),
        Err(e) => info!("Merger ran into an error: {} Shutting down.\n", e)
    }

    //Rejoin workers
    match evb_handle.join() {
        Ok(_) => info!("Successfully joined evb thread.\n"),
        Err(e) => error!("Error on joining evb thread: {:?}", e)
    }
    match hdf_handle.join() {
        Ok(_) => info!("Successfully joined hdf thread.\n"),
        Err(e) => error!("Error on joining hdf thread: {:?}", e)
    }

    return;

}
