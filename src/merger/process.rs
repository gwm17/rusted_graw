use std::sync::{Arc, Mutex};

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
    let hdf_path = config.get_hdf_file_name()?;

    log::info!("Configuration parsed.\n GRAW run path: {}\n HDF file path: {}\n Pad map file path: {}", run_path.display(), hdf_path.display(), config.pad_map_path.display());

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

    log::info!("Processing...");
    writer.write_fileinfo(&merger);
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
            writer.write_meta()?;
            flush_final_event(evb, writer)?;
            break;
        }
    }

    return Ok(())
}