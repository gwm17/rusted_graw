use std::fs::File;
use std::io::{Seek, Read, SeekFrom};
use std::path::{Path, PathBuf};

use super::error::EvtFileError;
use super::ring_item::RingItem;

// # EVT file
// .evt files contain the data recorded by the FRIB DAQ system
// The data is atomic in ring items that contain various types of data

#[allow(dead_code)]
#[derive(Debug)]
pub struct EvtFile {
    file_handle: File,
    file_path: PathBuf,
    size_bytes: u64,
//    next_item_metadata: ItemMetadata, // Store this to reduce read calls
    is_eof: bool,
    is_open: bool
}

impl EvtFile {

    /// Open a evt file in read-only mode.
    pub fn new(path: &Path) -> Result<Self, EvtFileError> {
        if !path.exists() {
            return Err(EvtFileError::BadFilePath(path.to_path_buf()));
        }

        let file_path = path.to_path_buf();
        let file = File::open(path)?;
        let size_bytes = file.metadata()?.len();
        let handle = file;

        Ok(EvtFile {file_handle: handle, file_path, size_bytes: size_bytes, is_eof: false, is_open: true })
    }

    pub fn get_next_item(&mut self) -> Result<Option<RingItem>, EvtFileError>  {
        let mut item = RingItem::new();
        let current_position: u64 = self.file_handle.stream_position()?;
        let mut sizebuf= [0,0,0,0];
        self.file_handle.read_exact(&mut sizebuf)?; // Get the ring item size (in bytes)
        item.size = u32::from_ne_bytes(sizebuf);
        self.file_handle.seek(SeekFrom::Start(current_position))?; // Go back to start of item (size is self contained)
        item.bytes = vec![0; item.size.try_into().unwrap()]; // set size of bytes vector
        match self.file_handle.read_exact(&mut item.bytes) { // try to read ring item
            Err(e) => match e.kind() {
                std::io::ErrorKind::UnexpectedEof => {
                    self.is_eof = true;
                    return Err(EvtFileError::EndOfFile);
                },
                _ => {
                    return Err(EvtFileError::IOError(e));
                }
            }
            Ok(()) => {
                return Ok(Some(item));
            }
        }
    }

}