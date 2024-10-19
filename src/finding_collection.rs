use crate::{
    as_mut_str_unchecked_no_borrow_check, as_str_unchecked_no_borrow_check,
    finding::{Finding, Precision, OUTPUT_BUF_LEN},
    helper::{starts_with_multibyte_char, SplitStr},
    input::{ByteCounter, INPUT_BUF_LEN},
    scanner::ScannerState,
};
use encoding_rs::DecoderResult;
use std::{
    io::Write,
    marker::PhantomPinned,
    ops::Deref,
    pin::Pin,
    slice, str,
    sync::{Arc, Mutex},
};

#[derive(Debug)]
pub struct FindingCollection<'a> {
    pub v: Vec<Finding<'a>>,
    pub first_byte_position: ByteCounter,
    output_buffer_bytes: Box<[u8]>,
    pub str_buf_overflow: bool,
    _marker: PhantomPinned,
}
impl FindingCollection<'_> {
    pub fn new(byte_offset: ByteCounter) -> Self {
        let output_buffer_bytes = Box::new([0u8; OUTPUT_BUF_LEN]);
        FindingCollection {
            v: Vec::new(),
            first_byte_position: byte_offset,
            output_buffer_bytes,
            str_buf_overflow: false,
            _marker: PhantomPinned,
        }
    }

    pub fn from<'a>(
        ss: Arc<Mutex<ScannerState>>,
        input_file_id: Option<u8>,
        input_buffer: &[u8],
        is_last_input_buffer: bool,
    ) -> Pin<Box<FindingCollection<'a>>> {
        let mut ss = ss.lock().unwrap();
        let mut fc = FindingCollection::new(ss.consumed_bytes);
        let mut extra_round = false;
        let mut decoder_input_start = 0usize;
        let mut decoder_input_end;
        let mut decoder_output_start = 0usize;
        let mut last_window_leftover_len = 0usize;
        if !ss.last_scan_run_leftover.is_empty() {
            fc.output_buffer_bytes
                [decoder_output_start..decoder_output_start + ss.last_scan_run_leftover.len()]
                .copy_from_slice(ss.last_scan_run_leftover.as_bytes());
            last_window_leftover_len = ss.last_scan_run_leftover.len();
            ss.last_scan_run_leftover.clear();
            decoder_output_start += last_window_leftover_len;
        }
        let mut last_window_str_was_printed_and_is_maybe_cut_str =
            ss.last_run_str_was_printed_and_is_maybe_cut_str;
        let decoder_input_window = 2 * ss.mission.output_line_char_nb_max;
        let mut is_last_window = false;
        '_input_window_loop: while decoder_input_start < input_buffer.len() {
            decoder_input_end = match decoder_input_start + decoder_input_window {
                n if n < input_buffer.len() => n,
                _ => {
                    is_last_window = true;
                    input_buffer.len()
                }
            };

            'decoder: loop {
                let output_buffer_slice: &mut str = as_mut_str_unchecked_no_borrow_check!(
                    &mut fc.output_buffer_bytes[decoder_output_start..]
                );
                let (decoder_result, decoder_read, decoder_written) =
                    ss.decoder.decode_to_str_without_replacement(
                        &input_buffer[decoder_input_start..decoder_input_end],
                        output_buffer_slice,
                        extra_round,
                    );

                let mut position_precision = Precision::Exact;
                if decoder_written > 0
                    && decoder_input_start == 0
                    && starts_with_multibyte_char(output_buffer_slice)
                {
                    let mut empty_decoder =
                        ss.decoder.encoding().new_decoder_without_bom_handling();
                    let mut buffer_bytes = [0u8; 8];
                    let buffer: &mut str = as_mut_str_unchecked_no_borrow_check!(buffer_bytes[..]);
                    let (_, _, written) = empty_decoder.decode_to_str_without_replacement(
                        input_buffer,
                        &mut *buffer,
                        true,
                    );
                    if (written == 0)
                        || (fc.output_buffer_bytes[0..written] != buffer_bytes[0..written])
                    {
                        position_precision = Precision::Before;
                    }
                }
                let mut split_str_start = decoder_output_start;
                let split_str_end = decoder_output_start + decoder_written;
                if last_window_leftover_len > 0 {
                    split_str_start -= last_window_leftover_len;
                    last_window_leftover_len = 0;
                    position_precision = Precision::Before;
                };
                let split_str_buffer = as_str_unchecked_no_borrow_check!(
                    fc.output_buffer_bytes[split_str_start..split_str_end]
                );
                let invalid_bytes_after_split_str_buffer = (decoder_result
                    != DecoderResult::InputEmpty
                    && decoder_result != DecoderResult::OutputFull)
                    || (is_last_window && is_last_input_buffer);
                let continue_str_if_possible = last_window_str_was_printed_and_is_maybe_cut_str;
                last_window_str_was_printed_and_is_maybe_cut_str = false;

                '_chunk_loop: for chunk in SplitStr::new(
                    split_str_buffer,
                    ss.mission.chars_min_nb,
                    ss.mission.require_same_unicode_block,
                    continue_str_if_possible,
                    invalid_bytes_after_split_str_buffer,
                    ss.mission.filter,
                    ss.mission.output_line_char_nb_max,
                ) {
                    if !chunk.s_is_to_be_filtered_again {
                        fc.v.push(Finding {
                            input_file_id,
                            mission: ss.mission.clone(),
                            position: ss.consumed_bytes + decoder_input_start as ByteCounter,
                            position_precision,
                            s: chunk.s,
                            s_completes_previous_s: chunk.s_completes_previous_s,
                        });

                        last_window_leftover_len = 0;

                        last_window_str_was_printed_and_is_maybe_cut_str = chunk.s_is_maybe_cut;
                    } else {
                        last_window_leftover_len = chunk.s.len();
                        last_window_str_was_printed_and_is_maybe_cut_str = false;
                    }
                    position_precision = Precision::After;
                }

                decoder_output_start += decoder_written;

                decoder_input_start += decoder_read;
                match decoder_result {
                    DecoderResult::InputEmpty => {
                        if is_last_window && is_last_input_buffer && !extra_round {
                            extra_round = true;
                        } else {
                            break 'decoder;
                        }
                    }
                    DecoderResult::OutputFull => {
                        fc.clear_and_mark_incomplete();
                        eprintln!("Buffer overflow. Output buffer is too small to receive all decoder data.\
                            Some findings got lost in input {:x}..{:x} from file {:?} for scanner ({})!",
                            ss.consumed_bytes,
                            ss.consumed_bytes + decoder_input_start as ByteCounter,
                            input_file_id,
                            char::from(ss.mission.mission_id + 97)
                        );
                        decoder_output_start = 0;
                        debug_assert!(
                        true,
                        "Buffer overflow. Output buffer is too small to receive all decoder data."
                    );
                    }
                    DecoderResult::Malformed(_, _) => {}
                };
            }
        }
        let last_window_leftover = as_str_unchecked_no_borrow_check!(
            fc.output_buffer_bytes
                [decoder_output_start - last_window_leftover_len..decoder_output_start]
        );
        ss.last_scan_run_leftover = String::from(last_window_leftover);
        ss.last_run_str_was_printed_and_is_maybe_cut_str =
            last_window_str_was_printed_and_is_maybe_cut_str;
        ss.consumed_bytes += decoder_input_start as ByteCounter;
        Box::pin(fc)
    }

    pub fn clear_and_mark_incomplete(&mut self) {
        self.v.clear();
        self.str_buf_overflow = true;
    }

    #[allow(dead_code)]
    pub fn print(&self, out: &mut dyn Write) -> crate::Result<()> {
        if self.str_buf_overflow {
            eprint!("Warning: output buffer overflow! Some findings might got lost.");
            eprintln!(
                "in input chunk 0x{:x}-0x{:x}.",
                self.first_byte_position,
                self.first_byte_position + INPUT_BUF_LEN as ByteCounter
            );
        }
        for finding in &self.v {
            finding.print(out)?;
        }
        Ok(())
    }
}

impl<'a> IntoIterator for &'a Pin<Box<FindingCollection<'a>>> {
    type Item = &'a Finding<'a>;
    type IntoIter = FindingCollectionIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        FindingCollectionIterator { fc: self, index: 0 }
    }
}

pub struct FindingCollectionIterator<'a> {
    fc: &'a FindingCollection<'a>,
    index: usize,
}

impl<'a> Iterator for FindingCollectionIterator<'a> {
    type Item = &'a Finding<'a>;
    fn next(&mut self) -> Option<&'a Finding<'a>> {
        let result = if self.index < self.fc.v.len() {
            Some(&self.fc.v[self.index])
        } else {
            None
        };
        self.index += 1;
        result
    }
}

impl<'a> Deref for FindingCollection<'a> {
    type Target = Vec<Finding<'a>>;

    fn deref(&self) -> &Self::Target {
        &self.v
    }
}
