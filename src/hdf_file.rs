use std::sync::mpsc::Receiver;
use std::path::Path;
use hdf5::File;

use crate::event::Event;

const GROUP_NAME: &str = "get";

#[allow(dead_code)]
#[derive(Debug)]
pub struct HDFWriter {
    event_queue: Receiver<Event>,
    file_handle: File,
    group: hdf5::Group
}

impl HDFWriter {

    pub fn new(path: &Path, event_queue: Receiver<Event>) -> Result<Self, hdf5::Error> {
        let file_handle = File::create(path)?;
        let group = file_handle.create_group(GROUP_NAME)?;
        Ok( Self {
            event_queue,
            file_handle,
            group
        } )
    }

    pub fn run(&self) -> Result<(), hdf5::Error> {

        loop {

            if let Ok(event) = self.event_queue.recv() {
                let builder =  self.group.new_dataset_builder();
                let event_id = event.event_id.to_string();
                builder.with_data(&event.convert_to_data_matrix())
                        .create(event_id.as_str())?;
            } else {
                break;
            }

        }

        Ok(())
    }
}