mod error;
pub use crate::error::{Error, Result};

mod read_xml;
mod write_xml;

mod epg;
pub use crate::epg::{Epg, EpgChannel, EpgEvent};
