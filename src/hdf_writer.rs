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
        let builder =  self.group.new_dataset_builder();
        let event_id = event.event_id.to_string();
        builder.with_data(&event.convert_to_data_matrix())
                .create(event_id.as_str())?;
        Ok(())
    }

}