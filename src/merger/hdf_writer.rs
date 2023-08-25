use std::os::unix::prelude::OsStrExt;
use std::path::Path;
use hdf5::{File, types::VarLenAscii};
use ndarray::Array1;
use std::error::Error;

use super::event::Event;
use super::merger::Merger;

const GROUP_NAME: &str = "get";
const META_NAME: &str = "meta";

/// # HDFWriter
/// A simple struct which wraps around the hdf5-rust library. Opens a file for writing and
/// can write Events.
#[allow(dead_code)]
#[derive(Debug)]
pub struct HDFWriter {
    file_handle: File, //Idk if this needs to be kept alive, but I think it does
    group: hdf5::Group,
    meta: hdf5::Group,
    meta_data: [u64;4]
}

impl HDFWriter {

    /// Create the writer, opening a file at path and creating the data group (named get)
    pub fn new(path: &Path) -> Result<Self, hdf5::Error> {
        let file_handle = File::create(path)?;
        let group = file_handle.create_group(GROUP_NAME)?;
        let meta = file_handle.create_group(META_NAME)?;
        let meta_data = [1000000000,0,0,0];
        Ok( Self {
            file_handle,
            group,
            meta,
            meta_data
        } )
    }

    /// Write an event, where the event is converted into a data matrix
    pub fn write_event(&mut self, event: Event) -> Result<(), hdf5::Error> {
        let header_builder =  self.group.new_dataset_builder();
        let body_builder =  self.group.new_dataset_builder();
        let event_body_name = format!("evt{}_data", event.event_id);
        let event_header_name = format!("evt{}_header", event.event_id);
        if u64::from(event.event_id) < self.meta_data[0] { // Catch first event
            self.meta_data[0] = u64::from(event.event_id);
            self.meta_data[1] = event.timestamp;
        }
        if u64::from(event.event_id) > self.meta_data[2] { // Catch last event
            self.meta_data[2] = u64::from(event.event_id);
            self.meta_data[3] = event.timestamp;
        }
        header_builder.with_data(&event.get_header_array())
                .create(event_header_name.as_str())?;
        body_builder.with_data(&event.convert_to_data_matrix())
                .create(event_body_name.as_str())?;
        Ok(())
    }

    /// Write graw file information in meta group
    pub fn write_fileinfo(&self, merger: &Merger) -> Result<(), Box<dyn Error>> {
        let file_stacks = merger.get_file_stacks();
        for stack in file_stacks.iter() {
            let file_builder = self.meta.new_dataset_builder();
            let size_builder = self.meta.new_dataset_builder();
            let file_name = format!("cobo{}asad{}_files", stack.get_cobo_number(), stack.get_asad_number());
            let size_name = format!("cobo{}asad{}_length", stack.get_cobo_number(), stack.get_asad_number());
            let file_stack = stack.get_file_stack_ref();
            let mut file_list = Array1::<VarLenAscii>::from_elem(file_stack.len()+1,VarLenAscii::from_ascii("".as_bytes())?);
            let mut size_list = Array1::<u64>::zeros([file_stack.len()+1]);
            size_list[0] = stack.get_active_file().get_size_bytes(); // Active file is the first one
            file_list[0] = VarLenAscii::from_ascii(stack.get_active_file().get_filename().to_str().unwrap().as_bytes())?;
            for (row,queue) in file_stack.iter().enumerate() {
                size_list[row+1] = queue.metadata().unwrap().len();
                file_list[row+1] = VarLenAscii::from_ascii(queue.as_path().file_name().unwrap().as_bytes())?;
            }
            size_builder.with_data(&size_list).create(size_name.as_str())?;
            file_builder.with_data(&file_list).create(file_name.as_str())?;
        }
        Ok(())
    }

    /// Write meta information on first and last events
    pub fn write_meta(&self) -> Result<(), hdf5::Error> {
        let meta_builder = self.meta.new_dataset_builder();
        let meta_name = format!("meta");
        meta_builder.with_data(&self.meta_data).create(meta_name.as_str())?;
        log::info!("{} events written from {} to {}", self.meta_data[2]-self.meta_data[0], self.meta_data[0], self.meta_data[2]);
        log::info!("Run lasted {} seconds", (self.meta_data[3]-self.meta_data[1])/100000000); // Time Stamp Clock is 100 MHz
        Ok(())
    }

}