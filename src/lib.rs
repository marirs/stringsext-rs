#![allow(clippy::too_many_arguments)]

pub mod error;
mod finding;
mod finding_collection;
mod helper;
mod input;
mod mission;
mod options;
mod scanner;

use crate::{finding_collection::FindingCollection, input::Slicer, scanner::ScannerStates};
use mission::Missions;
use scoped_threadpool::Pool;
use std::{
    path::PathBuf,
    pin::Pin,
    sync::{mpsc, Arc, Mutex},
};

pub type Result<T> = std::result::Result<T, error::Error>;

pub struct StringsScanner {
    scaner_states: ScannerStates,
}

impl StringsScanner {
    pub fn new(
        counter_offset: Option<&String>,
        encodings: &[String],
        chars_min: Option<&String>,
        same_unicode_block: bool,
        ascii_filter: Option<&String>,
        unicode_block_filter: Option<&String>,
        grep_char: Option<&String>,
        output_line_len: Option<&String>,
    ) -> Result<Self> {
        let missions = Missions::new(
            counter_offset,
            encodings,
            chars_min,
            same_unicode_block,
            ascii_filter,
            unicode_block_filter,
            grep_char,
            output_line_len,
        )?;
        let scaner_states = ScannerStates::new(missions);
        Ok(Self { scaner_states })
    }

    #[allow(unused_assignments)]
    pub fn run(&mut self, paths: Vec<PathBuf>) -> Result<Vec<String>> {
        let merger;
        let buf = Arc::new(Mutex::new(Vec::new()));
        let buff = buf.clone();
        let n_threads = self.scaner_states.len();
        {
            let (tx, rx) = mpsc::sync_channel::<Pin<Box<FindingCollection>>>(n_threads);
            merger = std::thread::spawn(move || {
                //                let mut output = Box::new(io::stdout()) as Box<dyn Write>;
                buf.lock().unwrap().push("\u{feff}".to_string());
                let mut stop = false;
                'batch_receiver: loop {
                    let mut results: Vec<Pin<Box<FindingCollection>>> =
                        Vec::with_capacity(n_threads);
                    for _ in 0..n_threads {
                        results.push(match rx.recv() {
                            Ok(fc) => fc,
                            _ => {
                                stop = true;
                                break 'batch_receiver;
                            }
                        });
                    }
                    let mut mm = buf.lock().unwrap();
                    //                    let mmm = mm.get_mut();
                    for finding in itertools::kmerge(&results) {
                        mm.push(finding.s.to_string());
                        //                        finding.print(mmm)?;
                    }
                    if stop {
                        break;
                    }
                }
                //                buf.lock().unwrap().write_all(&[b'\n'])?;
                //                buf.lock().unwrap().flush()?;
                Ok::<_, error::Error>(())
            });

            let input = Slicer::new(paths);
            let mut pool = Pool::new(self.scaner_states.len() as u32);
            for (slice, input_file_id, is_last_input_buffer) in input {
                pool.scoped(|scope| {
                    for ss in self.scaner_states.v.iter() {
                        let tx = tx.clone();
                        let slice = slice.clone();
                        scope.execute(move || {
                            let fc = FindingCollection::from(
                                ss.clone(),
                                input_file_id,
                                &slice,
                                is_last_input_buffer,
                            );
                            tx.send(fc).unwrap();
                        });
                    }
                });
            }
        }
        merger.join().unwrap()?;
        let ll = buff.lock().unwrap();
        //        let lll = ll.get_mut().drain(..);
        Ok(ll.to_vec())
    }
}
