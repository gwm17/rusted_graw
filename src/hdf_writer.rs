use std::path::Path;
use hdf5::File;

use crate::event::Event;

const GROUP_NAME: &str = "get";

/// # HDFWriter
/// A simple struct which wraps around the hdf5-rust library. Opens a file for writing and
/// can write Events.
#[allow(dead_code)]
#[derive(Debug)]
pub struct HDFWriter {
    file_handle: File, //Idk if this needs to be kept alive, but I think it does
    group: hdf5::Group
}

impl HDFWriter {

    /// Create the writer, opening a file at path and creating the data group (named get)
    pub fn new(path: &Path) -> Result<Self, hdf5::Error> {
        let file_handle = File::create(path)?;
        let group = file_handle.create_group(GROUP_NAME)?;
        Ok( Self {
            file_handle,
            group
        } )
    }

    /// Write an event, where the event is converted into a data matrix
    pub fn write_event(&self, event: Event) -> Result<(), hdf5::Error> {
        let header_builder =  self.group.new_dataset_builder();
        let body_builder =  self.group.new_dataset_builder();
        let event_body_name = format!("evt{}_data", event.event_id);
        let event_header_name = format!("evt{}_header", event.event_id);
        header_builder.with_data(&event.get_header_array())
                .create(event_header_name.as_str())?;
        body_builder.with_data(&event.convert_to_data_matrix())
                .create(event_body_name.as_str())?;
        Ok(())
    }

}