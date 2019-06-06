#[macro_use]
extern crate error_rules;

mod read_xml;
mod write_xml;

mod epg_event;
pub use crate::epg_event::EpgEvent;

mod epg_channel;
pub use crate::epg_channel::EpgChannel;

mod epg;
pub use crate::epg::{
    Epg,
    EpgError,
};


pub (crate) const FMT_DATETIME: &str = "%Y%m%d%H%M%S %z";
