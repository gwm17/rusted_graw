use std::fs::File;
use std::io::{BufReader, Seek, Read};
use std::path::Path;

use super::graw_frame::{FrameMetadata, GrawFrame, GrawFrameHeader};
use super::constants::*;
use super::error::GrawFileError;

const DEFAULT_BUFFER_SIZE: usize = 1_000_000; // 1MB buffer per file?

#[allow(dead_code)]
#[derive(Debug)]
pub struct GrawFile {
    file_handle: BufReader<File>,
    size_bytes: u64,
    next_frame_metadata: FrameMetadata,
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
        let handle = BufReader::with_capacity(DEFAULT_BUFFER_SIZE, file);

        Ok(GrawFile {  file_handle: handle, size_bytes: size_bytes, next_frame_metadata: FrameMetadata::default(), is_eof: false, is_open: true })
    }

    pub fn get_next_frame(&mut self) -> Result<GrawFrame, GrawFileError> {
        let next_header = self.get_next_frame_header()?;
        let frame_read_size: usize = (next_header.frame_size * SIZE_UNIT) as usize;
        let mut frame_word: Vec<u8> = vec![0; frame_read_size];

        //Clear metadata
        self.next_frame_metadata = FrameMetadata::default();

        //Check to see if we reach end of file... shouldn't happen here tho
        match self.file_handle.read_exact(&mut frame_word) {
            Err(e) => match e.kind() {
                std::io::ErrorKind::UnexpectedEof => {
                    self.is_eof = true;
                    return Err(GrawFileError::EndOfFile);
                },
                _ => {
                    return Err(GrawFileError::IOError(e));
                }
            }
            Ok(()) => {
                return Ok(GrawFrame::try_from(frame_word)?);
            }
        }
    }

    pub fn get_next_frame_metadata(&mut self) -> Result<&FrameMetadata, GrawFileError> {
        if self.next_frame_metadata == FrameMetadata::default() {
            self.next_frame_metadata = FrameMetadata::from(self.get_next_frame_header()?);
        }
        Ok(&self.next_frame_metadata)
    }

    pub fn is_eof(&self) -> &bool {
        &self.is_eof
    }

    #[allow(dead_code)]
    pub fn is_open(&self) -> &bool {
        &self.is_open
    }

    //Peek at the header of the next frame to extract sizing information or metadata
    //This resets the file stream to the position at the start of the header, as the read of the frame includes
    //reading the header
    fn get_next_frame_header(&mut self) -> Result<GrawFrameHeader, GrawFileError> {
        let read_size: usize = (EXPECTED_HEADER_SIZE as u32 * SIZE_UNIT) as usize;
        let current_position = self.file_handle.stream_position()?;
        let mut header_word: Vec<u8> = vec![0; read_size];
        //Check to see if we reach end of file
        match self.file_handle.read_exact(&mut header_word) {
            Err(e) => match e.kind() {
                std::io::ErrorKind::UnexpectedEof => {
                    self.is_eof = true;
                    return Err(GrawFileError::EndOfFile);
                }
                _ => {
                    return Err(GrawFileError::IOError(e));
                }
            }
            Ok(_) => ()
        }
        let (_, header) = GrawFrameHeader::read_from_buffer(&header_word)?;
        self.file_handle.seek(std::io::SeekFrom::Start(current_position))?;
        Ok(header)
    }

}
