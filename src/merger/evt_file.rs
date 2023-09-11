use std::fs::File;
use std::io::{Seek, Read, SeekFrom};
use std::path::{Path, PathBuf};

use byteorder::LittleEndian;
use byteorder::ReadBytesExt;

use super::error::EvtFileError;
use super::ring_item::RingItem;

/// # EvtFile
/// .evt files contain the data recorded by the FRIB DAQ system.
/// The data is atomic in RingItems that contain various types of data.
/// These RingItems can then be cast to functional types which parse the binary buffer
/// and allow the data to be accessed.

#[allow(dead_code)]
#[derive(Debug)]
pub struct EvtFile {
    file_handle: File,
    file_path: PathBuf,
    size_bytes: u64,
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

    /// Retrieve the next RingItem from the buffer. Returns a Result<Option<RingItem>>. If the Option is None,
    /// the file has run out of data.
    pub fn get_next_item(&mut self) -> Result<Option<RingItem>, EvtFileError>  {
        //First need to query the size of the next ring item.
        let current_position: u64 = self.file_handle.stream_position()?;
        let item_size = self.file_handle.read_u32::<LittleEndian>()? as usize;
        
        self.file_handle.seek(SeekFrom::Start(current_position))?; // Go back to start of item (size is self contained)
        let mut buffer: Vec<u8> = vec![0; item_size]; // set size of bytes vector
        match self.file_handle.read_exact(&mut buffer) { // try to read ring item
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
                return Ok(Some(RingItem::try_from(buffer)?));
            }
        }
    }

}