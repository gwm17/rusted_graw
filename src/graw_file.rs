use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use super::graw_frame::GrawFrame;
use super::error::GrawFileError;

#[derive(Debug, Clone, Default)]
pub struct FrameMetadata {
    event_id: u32,
    event_time: u32
}


#[derive(Debug)]
pub struct GrawFile {
    file_handle: BufReader<File>,
    size_bytes: u64,
    buffer_size_bytes: usize,
    is_eof: bool,
    is_open: bool
}

impl GrawFile {

    pub fn new(path: &Path) -> Result<Self, GrawFileError> {
        if !path.exists() {
            return Err(GrawFileError::BadFilePath(path.to_path_buf()));
        }

        let file = File::open(path)?;
        let size_bytes = file.metadata()?.len();
        let handle = BufReader::new(file);


        Ok(GrawFile {  file_handle: handle, size_bytes: size_bytes, buffer_size_bytes: 8000, is_eof: false, is_open: true })
    }

    pub fn read_frame(&mut self) -> Result<GrawFrame, GrawFileError> {
        todo!()
    }

    pub fn read_frame_metadata(&mut self) -> Result<FrameMetadata, GrawFileError> {
        todo!()
    }

}
