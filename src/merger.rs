use std::path::Path;


use crate::constants::{NUMBER_OF_COBOS, NUMBER_OF_ASADS};
use crate::error::AsadStackError;

use super::asad_stack::AsadStack;
use super::graw_frame::GrawFrame;
use super::error::MergerError;

/*
    Merger
    One of the three workers which comprise the application. Merger essentially performs a merge-sort operation on the data files
 */
#[derive(Debug)]
pub struct Merger {
    file_stacks: Vec<AsadStack>,
    current_event: u32,
    total_data_size_bytes: u64,
}

impl Merger {

    //Create a new merger. Requires the path to the graw data files, as well as the queue for transmitting frames
    pub fn new(graw_dir: &Path) -> Result<Self, MergerError> {

        let mut merger = Merger {
            file_stacks: Vec::new(),
            current_event: 0,
            total_data_size_bytes: 0,
        };

        //For every asad in every cobo, attempt to make a stack
        for cobo in 0..NUMBER_OF_COBOS {
            for asad in 0..NUMBER_OF_ASADS {
                match AsadStack::new(graw_dir, cobo as i32, asad as i32) {
                    Ok(stack) => {
                        merger.file_stacks.push(stack);
                    }
                    Err(AsadStackError::NoMatchingFiles) => {
                        continue;
                    }
                    Err(e) => {
                        return Err(MergerError::AsadError(e));
                    }
                }
            }
        }

        //Oops no files
        if merger.file_stacks.len() == 0 {
            return Err(MergerError::NoFilesError);
        }

        merger.total_data_size_bytes = merger.file_stacks.iter().fold(0, |sum, stack| sum + stack.get_stack_size_bytes());
        Ok(merger)
    }

    pub fn get_next_frame(&mut self) -> Result<GrawFrame, MergerError> {
        let mut end_count: usize = 0;
        loop  {
            for stack in self.file_stacks.iter_mut() {

                match stack.get_next_frame_metadata() {
                    Ok(meta) => {
                        if meta.event_id == self.current_event {
                            return Ok(stack.get_next_frame()?);
                        }
                    }
                    Err(AsadStackError::NoMoreFiles) => {
                        //end_of_stack.push(idx);
                        end_count += 1;
                        continue;
                    }
                    Err(e) => {
                        return Err(MergerError::AsadError(e));
                    }
                }
            }

            self.current_event += 1;

            if end_count >= self.file_stacks.len() {
                break Err(MergerError::EndOfMerge);
            }
        }
    }

    pub fn get_total_data_size(&self) -> &u64 {
        &self.total_data_size_bytes
    }

}