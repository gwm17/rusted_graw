use std::sync::{Arc, Mutex};

use crate::merger::evt_file::EvtFile;
use crate::merger::ring_item::{RingItem,RunInfo,Scalers,Physics};

use super::hdf_writer::HDFWriter;
use super::pad_map::PadMap;
use super::merger::Merger;
use super::event_builder::EventBuilder;
use super::constants::SIZE_UNIT;
use super::error::ProcessorError;
use super::config::Config;

fn flush_final_event(mut evb: EventBuilder, mut writer: HDFWriter) -> Result<(), hdf5::Error> {
    if let Some(event) = evb.flush_final_event() {
        writer.write_event(event)
    } else {
        Ok(())
    }
}

pub fn process_run(config: Config, progress: Arc<Mutex<f32>>) -> Result<(), ProcessorError> {

    let run_path = config.get_run_directory()?;
    let evt_name = config.get_evtrun()?;
    let hdf_path = config.get_hdf_file_name()?;

    log::info!("Configuration parsed.\n GRAW run path: {}\n EVT file: {}\nHDF file path: {}\n Pad map file path: {}", run_path.display(), evt_name.display(), hdf_path.display(), config.pad_map_path.display());

    let pad_map = PadMap::new(&config.pad_map_path)?;

    //Initialize the merger, event builder, and hdf writer
    let mut merger = Merger::new(&run_path)?;
    log::info!("Total run size: {}", human_bytes::human_bytes(*merger.get_total_data_size() as f64));
    let mut evb = EventBuilder::new(pad_map);
    let mut writer = HDFWriter::new(&hdf_path)?;


    let total_data_size = merger.get_total_data_size();
    let flush_frac: f32 = 0.01;
    let mut count =0;
    let flush_val = (*total_data_size as f64 * flush_frac as f64) as u64;

    log::info!("Now processing evt data...");
    let mut evt_file = EvtFile::new(&evt_name)?; // open evt file
    let mut run_info = RunInfo::new();
    let mut counter: u32 = 0;
    let mut scaler_counter: u32 = 0;
    let mut event_counter: u32 = 0;
    loop {
        if let Some(mut ring) = evt_file.get_next_item()? {
            let index: usize;
            let mut scalers = Scalers::new();
            let mut physics = Physics::new();
            if ring.bytes[8] == 20 { // ring header might or might not be present
                index = 28;
            } else {
                index = 12;
            }
            match ring.bytes[4] { // process each ring depending on its type
                1 => { // Begin run
                    RingItem::begin(&ring, index, &mut run_info);
                    log::info!("Detected begin run {}: {}", run_info.run, run_info.title);
                }
                2 => { // End run
                    RingItem::end(&ring, index, &mut run_info);
                    log::info!("Detected end run {} which lasted {} seconds", run_info.run, run_info.seconds);
                    writer.write_evtinfo(run_info)?;
                    break;
                }
                12 => RingItem::dummy(&ring),
                20 => { // Scalers
                    RingItem::scaler(&ring, index, &mut scalers);
                    writer.write_scalers(scalers, scaler_counter)?;
                    scaler_counter += 1;
                }
                30 => { // Physics data
                    RingItem::remove_boundaries(&mut ring, index); // physics event often cross VMUSB buffer boundary
                    RingItem::physics(&ring, index, &mut physics);
                    writer.write_physics(physics, &event_counter)?;
                    event_counter += 1;
                }
                31 => RingItem::counter(&ring, &mut counter),
                _ => log::info!("Unrecognized ring type: {}", ring.bytes[4])
            }
        } else {
            break;
        }
        
    }
    log::info!("Done with evt data.");

    log::info!("Processing get data...");
    writer.write_fileinfo(&merger).unwrap();
    loop {
        if let Some(frame) = merger.get_next_frame()? { //Merger found a frame
            //bleh
            count += (frame.header.frame_size as u32 * SIZE_UNIT) as u64;
            if count > flush_val {
                count = 0;
                if let Ok(mut bar) = progress.lock() {
                    *bar += flush_frac;
                }
            }

            if let Some(event) = evb.append_frame(frame)? {
                writer.write_event(event)?;
            } else {
                continue;
            }
        } else { //If the merger returns none, there is no more data to be read
            writer.write_meta()?; // write meta dataset (first and last event id + ts)
            flush_final_event(evb, writer)?;
            break;
        }
    }
    log::info!("Done with get data.");
    
    return Ok(())
}