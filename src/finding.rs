use crate::{input::ByteCounter, mission::Mission};
use std::{io::Write, str, sync::Arc};

#[cfg(not(test))]
pub const OUTPUT_BUF_LEN: usize = 0x9192;
#[cfg(test)]
pub const OUTPUT_BUF_LEN: usize = 0x40;

#[derive(Debug, Eq, PartialEq)]
pub enum Precision {
    Before,
    Exact,
    After,
}

#[derive(Debug)]
pub struct Finding<'a> {
    pub input_file_id: Option<u8>,
    pub mission: Arc<Mission>,
    pub position: ByteCounter,
    pub position_precision: Precision,
    pub s: &'a str,
    pub s_completes_previous_s: bool,
}

impl Eq for Finding<'_> {}

impl PartialEq for Finding<'_> {
    fn eq(&self, other: &Self) -> bool {
        (self.position == other.position)
            && (self.position_precision == other.position_precision)
            && (self.mission.encoding.name() == other.mission.encoding.name())
            && (self.mission.filter == other.mission.filter)
            && (self.s == other.s)
    }
}

impl PartialOrd for Finding<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.position != other.position {
            self.position.partial_cmp(&other.position)
        } else if self.mission.mission_id != other.mission.mission_id {
            self.mission
                .mission_id
                .partial_cmp(&other.mission.mission_id)
        } else if self.mission.filter.ubf != other.mission.filter.ubf {
            self.mission
                .filter
                .ubf
                .partial_cmp(&other.mission.filter.ubf)
        } else {
            self.mission.filter.af.partial_cmp(&other.mission.filter.af)
        }
    }
}

impl<'a> Finding<'a> {
    pub fn print(&self, out: &mut dyn Write) -> crate::Result<()> {
        out.write_all(b"\n")?;
        out.write_all(self.s.as_bytes())?;
        Ok(())
    }
}
