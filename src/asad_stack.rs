use std::path::{PathBuf, Path};
use std::collections::VecDeque;

use crate::graw_file::GrawFile;
use crate::error::{GrawFileError, AsadStackError};
use crate::graw_frame::{GrawFrame, FrameMetadata};


#[allow(dead_code)]
#[derive(Debug)]
pub struct AsadStack {
    active_file: GrawFile,
    file_stack: VecDeque<PathBuf>,
    cobo_number: i32,
    asad_number: i32,
    parent_path: PathBuf,
    total_stack_size_bytes: u64,
}

impl AsadStack {

    pub fn new(data_path: &Path, cobo_number: i32, asad_number: i32) -> Result<Self, AsadStackError> {
        let parent_path = data_path.join(format!("mm{}", cobo_number));

        let (mut file_stack, total_stack_size_bytes) = Self::get_file_stack(&parent_path, &cobo_number, &asad_number)?;
        if let Some(path) = file_stack.pop_front() {
            Ok(AsadStack { active_file: GrawFile::new(&path)?, file_stack, cobo_number, asad_number, parent_path, total_stack_size_bytes })
        } else {
            Err(AsadStackError::NoMatchingFiles)
        }
    }

    pub fn get_next_frame_metadata(&mut self) -> Result<FrameMetadata, AsadStackError> {
        loop {
            match self.active_file.get_next_frame_metadata() {
                Ok(meta) => return Ok(meta),
                Err(GrawFileError::EndOfFile) => {
                    self.move_to_next_file()?;
                    continue;
                }
                Err(e) => return Err(AsadStackError::FileError(e))
            }
        }
    }

    pub fn get_next_frame(&mut self) -> Result<GrawFrame, AsadStackError> {
        Ok(self.active_file.get_next_frame()?)
    }

    pub fn get_stack_size_bytes(&self) -> &u64 {
        &self.total_stack_size_bytes
    }

    fn get_file_stack(parent_path: &Path, cobo_number: &i32, asad_number: &i32) -> Result<(VecDeque<PathBuf>, u64), AsadStackError> {
        let stack: VecDeque<PathBuf>;
        let mut file_list: Vec<PathBuf> = Vec::new();
        let start_pattern = format!("CoBo{}_AsAd{}", *cobo_number, *asad_number);
        let end_pattern = ".graw";
        for item in parent_path.read_dir()? {
            let item_path = item?.path();
            let item_path_str = item_path.to_str().unwrap();
            if item_path_str.contains(&start_pattern) && item_path_str.contains(&end_pattern) {
                file_list.push(item_path);
            }
        }

        if file_list.len() == 0 {
            return Err(AsadStackError::NoMatchingFiles);
        }

        let total_stack_size_bytes = file_list.iter().fold(0, |sum, path| sum + path.metadata().unwrap().len());

        file_list.sort();
        stack = file_list.into();

        return Ok((stack, total_stack_size_bytes));
    }

    fn move_to_next_file(&mut self) -> Result<(), AsadStackError> {
        loop {
            if let Some(next_file_path) = self.file_stack.pop_front() {
                let next_file = GrawFile::new(&next_file_path)?;
                if *next_file.is_open() && !(*next_file.is_eof()) {
                    self.active_file = next_file;
                    return Ok(());
                }
            }
            else {
                return Err(AsadStackError::NoMoreFiles);
            }
        }
    }

}