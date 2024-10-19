#![allow(clippy::too_many_arguments, clippy::type_complexity, dead_code)]
use crate::{
    input::ByteCounter,
    options::{
        ASCII_ENC_LABEL, CHARS_MIN_DEFAULT, COUNTER_OFFSET_DEFAULT, ENCODING_DEFAULT,
        OUTPUT_LINE_CHAR_NB_MAX_DEFAULT, OUTPUT_LINE_CHAR_NB_MIN,
    },
};
use encoding_rs::*;
use std::{
    cmp::{self, Eq, Ord},
    fmt,
    ops::Deref,
    str::{self, FromStr},
    sync::Arc,
};

pub const UTF8_FILTER_ASCII_MODE_DEFAULT: Utf8Filter = Utf8Filter {
    af: AF_ALL & !AF_CTRL,
    ubf: UBF_NONE,
    grep_char: None,
};

pub const UTF8_FILTER_NON_ASCII_MODE_DEFAULT: Utf8Filter = Utf8Filter {
    af: AF_ALL & !AF_CTRL,
    ubf: UBF_COMMON,
    grep_char: None,
};

pub const UBF_ALL_VALID: u64 = UBF_ALL & !UBF_INVALID;
pub const UBF_ALL: u64 = 0xffff_ffff_ffff_ffff;
pub const UBF_NONE: u64 = 0x0000_0000_0000_0000;
pub const UBF_INVALID: u64 = 0xffe0_0000_0000_0003;
pub const UBF_LATIN: u64 = 0x0000_0000_0000_01fc;
pub const UBF_ACCENTS: u64 = 0x0000_0000_0000_3000;
pub const UBF_GREEK: u64 = 0x0000_0000_0000_C000;
pub const UBF_IPA: u64 = 0x0000_0000_0000_0700;
pub const UBF_CYRILLIC: u64 = 0x0000_0000_001f_0000;
pub const UBF_ARMENIAN: u64 = 0x0000_0000_0020_0000;
pub const UBF_HEBREW: u64 = 0x0000_0000_00c0_0000;
pub const UBF_ARABIC: u64 = 0x0000_0000_2f00_0000;
pub const UBF_SYRIAC: u64 = 0x0000_0000_1000_0000;
pub const UBF_AFRICAN: u64 = 0x0000_0000_ffe0_0000;
pub const UBF_COMMON: u64 = 0x0000_0000_ffff_fffc;
pub const UBF_KANA: u64 = 0x0000_0008_0000_0000;
pub const UBF_CJK: u64 = 0x0000_03f0_0000_0000;
pub const UBF_HANGUL: u64 = 0x0000_3800_0000_0000;
pub const UBF_ASIAN: u64 = 0x0000_3ffc_0000_0000;
pub const UBF_PUA: u64 = 0x0010_4000_0000_0000;
pub const UBF_MISC: u64 = 0x0000_8006_0000_0000;
pub const UBF_UNCOMMON: u64 = 0x000f_0000_0000_0000;

pub const UNICODE_BLOCK_FILTER_ALIASSE: [([u8; 12], u64, [u8; 25]); 18] = [
    (*b"African     ", UBF_AFRICAN, *b"all in U+540..U+800      "),
    (
        *b"All-Asian   ",
        UBF_ALL & !UBF_INVALID & !UBF_ASIAN,
        *b"all, except Asian        ",
    ),
    (
        *b"All         ",
        UBF_ALL & !UBF_INVALID,
        *b"all valid multibyte UTF-8",
    ),
    (
        *b"Arabic      ",
        UBF_ARABIC | UBF_SYRIAC,
        *b"Arabic+Syriac            ",
    ),
    (
        *b"Armenian    ",
        UBF_ARMENIAN,
        *b"Armenian                 ",
    ),
    (*b"Asian       ", UBF_ASIAN, *b"all in U+3000..U+E000    "),
    (*b"Cjk         ", UBF_CJK, *b"CJK: U+4000..U+A000      "),
    (*b"Common      ", UBF_COMMON, *b"all 2-byte-UFT-8         "),
    (
        *b"Cyrillic    ",
        UBF_CYRILLIC,
        *b"Cyrillic                 ",
    ),
    (
        *b"Default     ",
        UBF_ALL & !UBF_INVALID,
        *b"all valid multibyte UTF-8",
    ),
    (*b"Greek       ", UBF_GREEK, *b"Greek                    "),
    (*b"Hangul      ", UBF_HANGUL, *b"Hangul: U+B000..U+E000   "),
    (*b"Hebrew      ", UBF_HEBREW, *b"Hebrew                   "),
    (*b"Kana        ", UBF_KANA, *b"Kana: U+3000..U+4000     "),
    (
        *b"Latin       ",
        UBF_LATIN | UBF_ACCENTS,
        *b"Latin + accents          ",
    ),
    (*b"None        ", !UBF_ALL, *b"block all multibyte UTF-8"),
    (*b"Private     ", UBF_PUA, *b"private use areas        "),
    (
        *b"Uncommon    ",
        UBF_UNCOMMON | UBF_PUA,
        *b"private + all>=U+10_000  ",
    ),
];

pub const AF_ALL: u128 = 0xffff_ffff_ffff_ffff_ffff_ffff_ffff_fffe;
pub const AF_NONE: u128 = 0x0000_0000_0000_0000_0000_0000_0000_0000;
pub const AF_CTRL: u128 = 0x8000_0000_0000_0000_0000_0000_ffff_ffff;
pub const AF_WHITESPACE: u128 = 0x0000_0000_0000_0000_0000_0001_0000_1e00;
pub const AF_DEFAULT: u128 = AF_ALL & !AF_CTRL;

pub const ASCII_FILTER_ALIASSE: [([u8; 12], u128, [u8; 25]); 6] = [
    (*b"All         ", AF_ALL, *b"all ASCII = pass all     "),
    (
        *b"All-Ctrl    ",
        AF_ALL & !AF_CTRL,
        *b"all-control              ",
    ),
    (
        *b"All-Ctrl+Wsp",
        AF_ALL & !AF_CTRL | AF_WHITESPACE,
        *b"all-control+whitespace   ",
    ),
    (*b"Default     ", AF_DEFAULT, *b"all-control              "),
    (*b"None        ", AF_NONE, *b"block all 1-byte UTF-8   "),
    (
        *b"Wsp         ",
        AF_WHITESPACE,
        *b"only white-space         ",
    ),
];

#[derive(Eq, PartialEq, Copy, Clone)]
pub struct Utf8Filter {
    pub af: u128,
    pub ubf: u64,
    pub grep_char: Option<u8>,
}

impl Utf8Filter {
    #[inline]
    pub fn pass_af_filter(&self, b: u8) -> bool {
        debug_assert!(b & 0x80 == 0x00);
        1 << b & self.af != 0
    }
    #[inline]
    pub fn pass_ubf_filter(&self, b: u8) -> bool {
        debug_assert!(b & 0x80 == 0x80);
        1 << (b & 0x3f) & self.ubf != 0
    }
}

impl fmt::Debug for Utf8Filter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "af: 0x{:x}, ubf: 0x{:x}, grep_char: {:?}",
            self.af, self.ubf, self.grep_char
        )
    }
}

impl PartialOrd for Utf8Filter {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Utf8Filter {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        if self.ubf != other.ubf {
            self.ubf.cmp(&other.ubf)
        } else {
            (!self.af).cmp(&!other.af)
        }
    }
}

#[derive(Debug, Clone)]
pub struct Mission {
    pub mission_id: u8,
    pub counter_offset: ByteCounter,
    pub encoding: &'static Encoding,
    pub chars_min_nb: u8,
    pub require_same_unicode_block: bool,
    pub filter: Utf8Filter,
    pub output_line_char_nb_max: usize,
    pub print_encoding_as_ascii: bool,
}

#[derive(Debug)]
pub struct Missions {
    pub v: Vec<Arc<Mission>>,
}

impl Deref for Missions {
    type Target = Vec<Arc<Mission>>;
    fn deref(&self) -> &Self::Target {
        &self.v
    }
}

#[macro_export]
macro_rules! parse_integer {
    ($s:expr, $x_from_str_radix:expr, $x_from_str:expr) => {{
        match $s {
            Some(s) if s.is_empty() => None,
            Some(s) if s.trim().len() >= 2 && s.trim()[..2] == *"0x" => {
                Some($x_from_str_radix(&s.trim()[2..], 16)?)
            }
            Some(s) => Some($x_from_str(s.trim())?),
            None => None,
        }
    }};
}

#[macro_export]
macro_rules! parse_filter_parameter {
    ($s:expr, $x_from_str_radix:expr, $list:expr) => {{
        match $s {
            Some(s) if s.trim().len() >= 2 && s.trim()[..2] == *"0x" => {
                Some($x_from_str_radix(&s.trim()[2..], 16)?)
            }
            Some(s) if s.is_empty() => None,
            Some(s) => {
                let s = s.trim();
                let mut oubf = None;
                for (ubf_name, ubf, _) in $list.iter() {
                    if s.len() <= ubf_name.len() && *s.as_bytes() == ubf_name[..s.len()] {
                        oubf = Some(*ubf);
                        break;
                    };
                }
                if oubf.is_some() {
                    oubf
                } else {
                    return Err($crate::error::Error::InvalidFilterName(s.to_string()));
                }
            }
            None => None,
        }
    }};
}

impl Missions {
    pub fn new(
        flag_counter_offset: Option<&String>,
        flag_encoding: &[String],
        flag_chars_min_nb: Option<&String>,
        flag_same_unicode_block: bool,
        flag_ascii_filter: Option<&String>,
        flag_unicode_block_filter: Option<&String>,
        flag_grep_char: Option<&String>,
        flag_output_line_len: Option<&String>,
    ) -> crate::Result<Self> {
        let flag_counter_offset = parse_integer!(
            flag_counter_offset,
            ByteCounter::from_str_radix,
            ByteCounter::from_str
        );

        let flag_chars_min_nb = parse_integer!(flag_chars_min_nb, u8::from_str_radix, u8::from_str);

        let flag_ascii_filter = parse_filter_parameter!(
            flag_ascii_filter,
            u128::from_str_radix,
            ASCII_FILTER_ALIASSE
        );

        let flag_unicode_block_filter = parse_filter_parameter!(
            flag_unicode_block_filter,
            u64::from_str_radix,
            UNICODE_BLOCK_FILTER_ALIASSE
        );

        let flag_grep_char = parse_integer!(flag_grep_char, u8::from_str_radix, u8::from_str);
        if let Some(m) = flag_grep_char {
            if m > 127 {
                return Err(crate::error::Error::GrepChar(m));
            }
        }

        let flag_output_line_len =
            parse_integer!(flag_output_line_len, usize::from_str_radix, usize::from_str);
        if let Some(m) = flag_output_line_len {
            if m < OUTPUT_LINE_CHAR_NB_MIN {
                return Err(crate::error::Error::MinimumOutputLineLength(
                    OUTPUT_LINE_CHAR_NB_MIN,
                    m,
                ));
            }
        }

        let mut v = Vec::new();
        let encoding_default: &[String; 1] = &[ENCODING_DEFAULT.to_string()];

        let enc_iter = if flag_encoding.is_empty() {
            encoding_default.iter()
        } else {
            flag_encoding.iter()
        };

        for (mission_id, enc_opt) in enc_iter.enumerate() {
            let (enc_name, chars_min_nb, filter_af, filter_ubf, filter_grep_char) =
                Self::parse_enc_opt(enc_opt)?;

            let mut enc_name = match enc_name {
                Some(s) => s,
                None => ENCODING_DEFAULT,
            };

            let counter_offset = match flag_counter_offset {
                Some(n) => n,
                None => COUNTER_OFFSET_DEFAULT,
            };

            let chars_min_nb = match chars_min_nb {
                Some(n) => n,
                None => match flag_chars_min_nb {
                    Some(n) => n,
                    None => CHARS_MIN_DEFAULT,
                },
            };

            let require_same_unicode_block = flag_same_unicode_block;

            let output_line_char_nb_max = match flag_output_line_len {
                Some(n) => n,
                None => OUTPUT_LINE_CHAR_NB_MAX_DEFAULT,
            };

            if output_line_char_nb_max < OUTPUT_LINE_CHAR_NB_MIN {
                return Err(crate::error::Error::ScannerMinimumOutputLineLength(
                    char::from((mission_id + 97) as u8).to_string(),
                    OUTPUT_LINE_CHAR_NB_MIN,
                    output_line_char_nb_max,
                ));
            }

            let filter_af = filter_af.unwrap_or_else(|| {
                flag_ascii_filter.unwrap_or(if enc_name == ASCII_ENC_LABEL {
                    UTF8_FILTER_ASCII_MODE_DEFAULT.af
                } else {
                    UTF8_FILTER_NON_ASCII_MODE_DEFAULT.af
                })
            });

            let filter_ubf = filter_ubf.unwrap_or_else(|| {
                flag_unicode_block_filter.unwrap_or(if enc_name == ASCII_ENC_LABEL {
                    UTF8_FILTER_ASCII_MODE_DEFAULT.ubf
                } else {
                    UTF8_FILTER_NON_ASCII_MODE_DEFAULT.ubf
                })
            });

            let filter_grep_char = match filter_grep_char {
                Some(f) => Some(f),
                None => match flag_grep_char {
                    Some(f) => Some(f),
                    None => {
                        if enc_name == ASCII_ENC_LABEL {
                            UTF8_FILTER_ASCII_MODE_DEFAULT.grep_char
                        } else {
                            UTF8_FILTER_NON_ASCII_MODE_DEFAULT.grep_char
                        }
                    }
                },
            };

            if let Some(m) = filter_grep_char {
                if m > 127 {
                    return Err(crate::error::Error::ScanerGrepCode(
                        char::from((mission_id + 97) as u8).to_string(),
                        m,
                    ));
                }
            }

            let filter = Utf8Filter {
                af: filter_af,
                ubf: filter_ubf,
                grep_char: filter_grep_char,
            };

            let mut print_encoding_as_ascii = false;
            if enc_name == ASCII_ENC_LABEL {
                print_encoding_as_ascii = true;
                enc_name = "x-user-defined"
            };

            let encoding = &Encoding::for_label(enc_name.as_bytes())
                .ok_or(crate::error::Error::Encoding(enc_name.to_string()))?;

            v.push(Arc::new(Mission {
                counter_offset,
                encoding,
                chars_min_nb,
                require_same_unicode_block,
                filter,
                output_line_char_nb_max,
                mission_id: mission_id as u8,
                print_encoding_as_ascii,
            }));
        }

        Ok(Missions { v })
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.v.len()
    }

    #[inline]
    fn parse_enc_opt(
        enc_opt: &str,
    ) -> crate::Result<(
        Option<&str>,
        Option<u8>,
        Option<u128>,
        Option<u64>,
        Option<u8>,
    )> {
        let mut i = enc_opt.split_terminator(',');

        let enc_name = match i.next() {
            Some("") => None,
            Some(s) => Some(s.trim()),
            None => None,
        };

        let chars_min_nb = parse_integer!(i.next(), u8::from_str_radix, u8::from_str);

        let filter_af =
            parse_filter_parameter!(i.next(), u128::from_str_radix, ASCII_FILTER_ALIASSE);

        let filter_ubf =
            parse_filter_parameter!(i.next(), u64::from_str_radix, UNICODE_BLOCK_FILTER_ALIASSE);

        let grep_char = parse_integer!(i.next(), u8::from_str_radix, u8::from_str);

        if i.next().is_some() {
            return Err(crate::error::Error::TooManyEncodings(enc_opt.to_string()));
        }
        Ok((enc_name, chars_min_nb, filter_af, filter_ubf, grep_char))
    }
}
