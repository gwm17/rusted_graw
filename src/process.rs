use indicatif::{ProgressBar, ProgressStyle};

use crate::hdf_writer::HDFWriter;
use crate::merger::Merger;
use crate::event_builder::EventBuilder;
use crate::constants::SIZE_UNIT;
use crate::error::ProcessorError;

fn flush_final_event(mut evb: EventBuilder, writer: HDFWriter) -> Result<(), hdf5::Error> {
    if let Some(event) = evb.flush_final_event() {
        writer.write_event(event)
    } else {
        Ok(())
    }
}

pub fn process_run(mut merger: Merger, mut evb: EventBuilder, writer: HDFWriter) -> Result<(), ProcessorError> {

        let progress = ProgressBar::new(*merger.get_total_data_size());
        let style = ProgressStyle::with_template("[{elapsed}] {bar:40.cyan/blue} {bytes}/{total_bytes} {msg}").unwrap();
        progress.set_style(style);

        let total_data_size = merger.get_total_data_size();
        let flush_frac = 0.01;
        let mut count =0;
        let flush_val = (*total_data_size as f64 * flush_frac) as u64;

        loop {
            if let Some(frame) = merger.get_next_frame()? { //Merger found a frame
                //bleh
                count += (frame.header.frame_size as u32 * SIZE_UNIT) as u64;
                if count > flush_val {
                    progress.inc(count);
                    count = 0;
                }

                if let Some(event) = evb.append_frame(frame)? {
                    writer.write_event(event)?;
                } else {
                    continue;
                }
            } else { //If the merger returns none, there is no more data to be read
                flush_final_event(evb, writer)?;
                break;
            }
        }

        progress.finish();

        return Ok(())
}