use std::sync::{Arc, Mutex};
use std::path::PathBuf;

use crate::merger::evt_file::EvtFile;
use crate::merger::ring_item::{RunInfo, RingType, BeginRunItem, EndRunItem, PhysicsItem, ScalersItem, CounterItem};

use super::hdf_writer::HDFWriter;
use super::pad_map::PadMap;
use super::merger::Merger;
use super::event_builder::EventBuilder;
use super::constants::SIZE_UNIT;
use super::error::ProcessorError;
use super::config::Config;

fn flush_final_event(mut evb: EventBuilder, mut writer: HDFWriter, event_counter: &u64) -> Result<(), hdf5::Error> {
    if let Some(event) = evb.flush_final_event() {
        writer.write_event(event, &event_counter)
    } else {
        Ok(())
    }
}

fn process_evt_data(evt_path: PathBuf, writer: &HDFWriter) -> Result<(), ProcessorError> {
    let mut evt_file = EvtFile::new(&evt_path)?; // open evt file
    let mut run_info = RunInfo::new();
    let mut scaler_counter: u32 = 0;
    let mut event_counter = CounterItem::new();
    loop {
        if let Some(mut ring) = evt_file.get_next_item()? {
            match ring.ring_type { // process each ring depending on its type
                RingType::BeginRun => { // Begin run
                    run_info.begin = BeginRunItem::try_from(ring)?;
                    log::info!("Detected begin run -- {}", run_info.print_begin());
                }
                RingType::EndRun => { // End run
                    run_info.end = EndRunItem::try_from(ring)?;
                    log::info!("Detected end run -- {}", run_info.print_end());
                    writer.write_evtinfo(run_info)?;
                    break;
                }
                RingType::Dummy => (),
                RingType::Scalers => { // Scalers
                    writer.write_scalers(ScalersItem::try_from(ring)?, scaler_counter)?;
                    scaler_counter += 1;
                }
                RingType::Physics => { // Physics data
                    ring.remove_boundaries(); // physics event often cross VMUSB buffer boundary
                    writer.write_physics(PhysicsItem::try_from(ring)?, &event_counter.count)?;
                    event_counter.count += 1;
                }
                RingType::Counter => {
                    event_counter = CounterItem::try_from(ring)?;
                },
                _ => log::info!("Unrecognized ring type: {}", ring.bytes[4])
            }
        } else {
            break;
        }
    }
    Ok(())
}

pub fn process_run(config: Config, progress: Arc<Mutex<f32>>) -> Result<(), ProcessorError> {

    let evt_path = config.get_evtrun()?;
    let hdf_path = config.get_hdf_file_name()?;
    let pad_map = PadMap::new(&config.pad_map_path)?;

    //Initialize the merger, event builder, and hdf writer
    let mut merger = Merger::new(&config)?;
    log::info!("Total run size: {}", human_bytes::human_bytes(*merger.get_total_data_size() as f64));
    let mut evb = EventBuilder::new(pad_map);
    let mut writer = HDFWriter::new(&hdf_path)?;


    let total_data_size = merger.get_total_data_size();
    let flush_frac: f32 = 0.01;
    let mut count =0;
    let flush_val = (*total_data_size as f64 * flush_frac as f64) as u64;

    log::info!("Now processing evt data...");
    process_evt_data(evt_path, &writer)?;
    log::info!("Done with evt data.");

    log::info!("Processing get data...");
    writer.write_fileinfo(&merger).unwrap();
    let mut event_counter = 0;
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
                writer.write_event(event, &event_counter)?;
                event_counter += 1;
            } else {
                continue;
            }
        } else { //If the merger returns none, there is no more data to be read
            writer.write_meta()?; // write meta dataset (first and last event id + ts)
            flush_final_event(evb, writer, &event_counter)?;
            break;
        }
    }
    log::info!("Done with get data.");
    
    return Ok(())
}