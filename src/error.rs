use std::fmt::Display;
use std::path::PathBuf;
use std::error::Error;

use super::constants::*;

/*
    GrawData errors
 */
#[derive(Debug, Clone)]
pub enum GrawDataError {
    BadAgetID(u8),
    BadChannel(u8),
    BadTimeBucket(u16),
}

impl Display for GrawDataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GrawDataError::BadAgetID(id) => write!(f, "Invaild aget ID {} found in GrawData!", id),
            GrawDataError::BadChannel(chan) => write!(f, "Invalid channel {} found in GrawData!", chan),
            GrawDataError::BadTimeBucket(bucket) => write!(f, "Invalid time bucket {} found in GrawData!", bucket)
        }
    }
}

impl Error for GrawDataError {

}

/*
    GrawFrame errors
 */
#[derive(Debug, Clone)]
pub enum GrawFrameError {
    ParsingError,
    IncorrectMetaType(u8),
    IncorrectFrameSize(u32, u32),
    IncorrectFrameType(u16),
    IncorrectHeaderSize(u16),
    IncorrectItemSize(u16),
    IncorrectNumberOfItems(u32, u32),
    BadDatum(GrawDataError)
}

impl From<GrawDataError> for GrawFrameError {
    fn from(value: GrawDataError) -> Self {
        GrawFrameError::BadDatum(value)
    }
}

impl Display for GrawFrameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GrawFrameError::ParsingError => write!(f, "Error parsing buffer into GrawFrame!"),
            GrawFrameError::IncorrectMetaType(t) => write!(f, "Incorrect meta type found for GrawFrame! Found: {} Expected: {}", t, EXPECTED_META_TYPE),
            GrawFrameError::IncorrectFrameSize(s, cs) => write!(f, "Incorrect frame size found for GrawFrame! Found: {}, Expected: {}", s, cs),
            GrawFrameError::IncorrectFrameType(t) => write!(f, "Incorrect frame type found for GrawFrame! Found: {}, Expected: {} or {}", t, EXPECTED_FRAME_TYPE_FULL, EXPECTED_FRAME_TYPE_PARTIAL),
            GrawFrameError::IncorrectHeaderSize(s) => write!(f, "Incorrect header size found for GrawFrame! Found: {}, Expected: {}", s, EXPECTED_HEADER_SIZE),
            GrawFrameError::IncorrectItemSize(s) => write!(f, "Incorrect item size found for GrawFrame! Found: {}, Expected: {} or {}", s, EXPECTED_ITEM_SIZE_FULL, EXPECTED_ITEM_SIZE_PARTIAL),
            GrawFrameError::IncorrectNumberOfItems(s, cs) => write!(f, "Incorrect number of items in GrawFrame! Found: {}, Expected: {}", s, cs),
            GrawFrameError::BadDatum(e) => write!(f, "Bad datum found in GrawFrame! Error: {}", e)
        }
    }
}

impl Error for GrawFrameError {

}


/*
    GrawFile errors
 */

#[derive(Debug)]
pub enum GrawFileError {
    BadFrame(GrawFrameError),
    BadFilePath(PathBuf),
    EndOfFile,
    IOError(std::io::Error)
}

impl From<GrawFrameError> for GrawFileError {
    fn from(value: GrawFrameError) -> Self {
        GrawFileError::BadFrame(value)
    }
}

impl From<std::io::Error> for GrawFileError {
    fn from(value: std::io::Error) -> Self {
        GrawFileError::IOError(value)
    }
}

impl Display for GrawFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GrawFileError::BadFrame(frame) => write!(f, "Bad frame found when reading GrawFile! Error: {}", frame),
            GrawFileError::BadFilePath(path) => write!(f, "File {} does not exist at GrawFile::new!", path.display()),
            GrawFileError::EndOfFile => write!(f, "File reached end!"),
            GrawFileError::IOError(e) => write!(f, "GrawFile recieved an io error: {}!", e)
        }
    }
}

impl Error for GrawFileError {

}