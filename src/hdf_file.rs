use std::path::Path;
use hdf5::File;

use crate::event::Event;

const GROUP_NAME: &str = "get";

/*
    HDFWriter
    One of the three workers which comprise the application. HDFWriter recieves completed Events and
    writes them to an HDF5 file
 */
#[allow(dead_code)]
#[derive(Debug)]
pub struct HDFWriter {
    file_handle: File,
    group: hdf5::Group
}

impl HDFWriter {

    //Create the writer, opening a file at path and creating the data group
    pub fn new(path: &Path) -> Result<Self, hdf5::Error> {
        let file_handle = File::create(path)?;
        let group = file_handle.create_group(GROUP_NAME)?;
        Ok( Self {
            file_handle,
            group
        } )
    }

    pub fn write_event(&self, event: Event) -> Result<(), hdf5::Error> {
        let builder =  self.group.new_dataset_builder();
        let event_id = event.event_id.to_string(); //Name by event id?
        builder.with_data(&event.convert_to_data_matrix())
                .create(event_id.as_str())?;
        Ok(())
    }

}