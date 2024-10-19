use crate::as_mut_slice_no_borrow_check;
use std::{
    fs::File,
    io::{self, Read},
    iter::Peekable,
    path::{Path, PathBuf},
    slice,
    vec::IntoIter,
};

pub type ByteCounter = u64;

#[cfg(not(test))]
pub const INPUT_BUF_LEN: usize = 4096;

#[cfg(test)]
pub const INPUT_BUF_LEN: usize = 0x20;

pub struct Slicer {
    filename_iter: Peekable<IntoIter<PathBuf>>,
    reader: Box<dyn Read>,
    current_input_idx: usize,
    current_input_is_last: bool,
    input_buffer: [u8; INPUT_BUF_LEN],
}

impl Slicer {
    #[inline]
    pub fn new(inputs: Vec<PathBuf>) -> Self {
        let mut filename_iter = inputs.into_iter().peekable();
        let filename = filename_iter.next().unwrap();
        let reader = match File::open(Path::new(&filename)) {
            Ok(file) => Box::new(file) as Box<dyn Read>,
            Err(e) => {
                eprintln!("Error: can not read file`{:?}`: {}", filename, e);
                Box::new(io::empty()) as Box<dyn Read>
            }
        };
        let current_input_is_last = filename_iter.peek().is_none();

        Self {
            filename_iter,
            reader,
            current_input_idx: 1,
            current_input_is_last,
            input_buffer: [0u8; INPUT_BUF_LEN],
        }
    }
}

impl Iterator for Slicer {
    type Item = (Vec<u8>, Option<u8>, bool);
    fn next(&mut self) -> Option<Self::Item> {
        let input_buffer_slice = as_mut_slice_no_borrow_check!(self.input_buffer);
        let no_bytes_received = self.reader.read(input_buffer_slice).unwrap_or_else(|_| {
            panic!(
                "Error: Could not read input stream no. {}",
                self.current_input_idx
            )
        });
        let result = input_buffer_slice[..no_bytes_received].to_vec();
        let this_stream_ended = no_bytes_received == 0;
        let input_ended = self.current_input_is_last && this_stream_ended;

        if this_stream_ended {
            if self.current_input_is_last {
                return None;
            } else {
                let filename = self.filename_iter.next().unwrap();
                self.current_input_idx += 1;
                self.current_input_is_last = self.filename_iter.peek().is_none();
                let reader = match File::open(Path::new(&filename)) {
                    Ok(file) => Box::new(file) as Box<dyn Read>,
                    Err(e) => {
                        eprintln!("Error: can not read file: {}", e);
                        Box::new(io::empty()) as Box<dyn Read>
                    }
                };
                self.reader = reader;
            }
        };

        let current_file_id = match self.current_input_idx {
            0 => None,
            c => Some(c as u8),
        };
        Some((result, current_file_id, input_ended))
    }
}
