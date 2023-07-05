use std::sync::mpsc::Sender;
use std::path::Path;

use super::graw_file::GrawFile;
use super::graw_frame::GrawFrame;
use super::error::MergerError;

#[derive(Debug)]
pub struct Merger {
    files: Vec<GrawFile>,
    frame_queue: Sender<GrawFrame>
}

impl Merger {
    pub fn new(graw_dir: &Path, queue: Sender<GrawFrame>) -> Result<Self, MergerError> {

        let mut merger = Merger {
            files: Vec::new(),
            frame_queue: queue
        };

        for item in graw_dir.read_dir()? {
            let filepath = item?.path();
            match filepath.extension() {
                Some(ext) => {
                    if ext == "graw" {
                        merger.files.push(GrawFile::new(&filepath)?);
                    }
                }
                _ => ()
            }
        }

        if merger.files.len() == 0 {
            return Err(MergerError::NoFilesError);
        }

        Ok(merger)
    }

    pub fn run(&mut self) -> Result<(), MergerError> {

        let mut event:u32 = 0; //what event are we on
        let mut eof_vec: Vec<usize> = vec![]; //list of files which went eof in the most recent pass
        loop {

            eof_vec.clear();

            for (idx, file) in self.files.iter_mut().enumerate() {
                if file.get_next_frame_metadata()?.event_id == event && !(*file.is_eof()) {
                    match self.frame_queue.send(file.get_next_frame()?) {
                        Ok(()) => (),
                        Err(_) => {
                            return Err(MergerError::SendError);
                        }
                    }
                }
                else if *file.is_eof() {
                    eof_vec.push(idx)
                }
            }

            for idx in eof_vec.iter() {
                self.files.swap_remove(*idx); //I don't think we care about file order?
            }

            if self.files.len() == 0 {
                break;
            }

            event += 1;
        }

        Ok(())
    }
}