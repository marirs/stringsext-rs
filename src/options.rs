use crate::input::ByteCounter;
use std::str::FromStr;

pub const ASCII_ENC_LABEL: &str = "ascii";
pub const ENCODING_DEFAULT: &str = "UTF-8";
pub const CHARS_MIN_DEFAULT: u8 = 4;
pub const COUNTER_OFFSET_DEFAULT: ByteCounter = 0;
pub const OUTPUT_LINE_CHAR_NB_MAX_DEFAULT: usize = 64;
pub const OUTPUT_LINE_CHAR_NB_MIN: usize = 6;

#[derive(Debug, Hash, Clone, Eq, PartialEq, Copy)]
pub enum Radix {
    O,
    X,
    D,
}

impl FromStr for Radix {
    type Err = String;
    fn from_str(rad: &str) -> Result<Radix, Self::Err> {
        match &*rad.to_ascii_lowercase() {
            "o" => Ok(Radix::O),
            "x" => Ok(Radix::X),
            "d" => Ok(Radix::D),
            _ => Err(String::from("can not convert radix variant")),
        }
    }
}
