use std::io;
use std::collections::HashMap;

use mpegts::psi::*;
use chrono::prelude::*;

use xml::reader::ParserConfig;
use xml::writer::EmitterConfig;

use parse_xml::parse_xml_tv;
use assemble_xml::assemble_xml_tv;

pub const FMT_DATETIME: &str = "%Y%m%d%H%M%S %z";

#[derive(Default, Debug, Clone)]
pub struct EpgEvent {
    /// Event start time
    pub start: i64,
    /// Event stop tiem (equal to the next event start time)
    pub stop: i64,
    /// Event title list
    pub title: HashMap<String, String>,
    /// Event short description list
    pub subtitle: HashMap<String, String>,
    /// Event description list
    pub desc: HashMap<String, String>,
}

impl EpgEvent {
    pub fn parse_eit(eit_item: &EitItem) -> EpgEvent {
        let mut event = EpgEvent::default();

        event.start = eit_item.start;
        event.stop = eit_item.start + i64::from(eit_item.duration);

        for desc in eit_item.descriptors.iter() {
            match desc {
                Descriptor::Desc4D(v) => {
                    if !v.name.is_empty() {
                        event.title
                            .entry(v.lang.to_string())
                            .or_insert_with(String::new)
                            .push_str(v.name.as_str());
                    }

                    if !v.text.is_empty() {
                        event.subtitle
                            .entry(v.lang.to_string())
                            .or_insert_with(String::new)
                            .push_str(v.text.as_str());
                    }
                },
                Descriptor::Desc4E(v) => {
                    if !v.text.is_empty() {
                        event.desc
                            .entry(v.lang.to_string())
                            .or_insert_with(String::new)
                            .push_str(v.text.as_str());
                    }
                },
                _ => (),
            };
        }

        event
    }

    pub fn assemble_eit(&self, codepage: usize) -> EitItem {
        let mut eit_item = EitItem::default();

        eit_item.start = self.start;
        eit_item.duration = (self.stop - self.start) as i32;
        eit_item.status = 1;

        for (lang, title) in &self.title {
            let subtitle = match self.subtitle.get(lang) {
                Some(v) => v,
                None => "",
            };

            eit_item.descriptors.push(Descriptor::Desc4D(Desc4D {
                lang: lang.to_string(),
                name: title.to_string(),
                text: subtitle.to_string(),
                codepage,
            }));
        }

        for (lang, desc) in &self.desc {
            eit_item.descriptors.push(Descriptor::Desc4E(Desc4E {
                number: 0,
                last_number: 0,
                lang: lang.to_string(),
                items: Vec::new(),
                text: desc.to_string(),
                codepage,
            }));
        }

        eit_item
    }
}

#[derive(Default, Debug)]
pub struct EpgChannel {
    /// Channel names list
    pub name: HashMap<String, String>,
    /// Channel events list
    pub events: Vec<EpgEvent>,
    /// First event identifier
    pub first_event_id: u16,
    /// Start time for last event
    pub last_event_start: i64,
}

impl EpgChannel {
    pub fn parse_eit(&mut self, eit: &Eit) {
        for eit_item in &eit.items {
            self.events.push(EpgEvent::parse_eit(eit_item));
        }

        self.sort();
    }

    pub fn sort(&mut self) {
        self.events.sort_by(|a, b| a.start.cmp(&b.start));

        if let Some(event) = self.events.last() {
            self.last_event_start = event.start;
        } else {
            self.last_event_start = 0;
        }
    }

    pub fn assemble_eit(&self, codepage: usize) -> Eit {
        let mut eit = Eit::default();
        eit.table_id = 0x50;

        let current_time = Utc::now().timestamp();
        let mut event_id = self.first_event_id;

        for event in &self.events {
            let mut eit_item = event.assemble_eit(codepage);
            eit_item.event_id = event_id;
            event_id += 1;
            if current_time >= event.start && current_time < event.stop {
                eit_item.status = 4;
            }
            eit.items.push(eit_item);
        }

        eit
    }
}

#[derive(Default, Debug)]
pub struct Epg {
    pub channels: HashMap<String, EpgChannel>,
}

impl Epg {
    pub fn parse_xml<R: io::Read>(&mut self, src: R) -> Result<(), String> {
        let mut reader = ParserConfig::new()
            .trim_whitespace(true)
            .ignore_comments(true)
            .create_reader(src)
            .into_iter();

        if let Err(e) = parse_xml_tv(self, &mut reader) {
            Err(format!("{}", e))
        } else {
            for channel in self.channels.values_mut() {
                channel.sort();
            }
            Ok(())
        }
    }

    pub fn assemble_xml<W: io::Write>(&self, dst: W) -> Result<(), String> {
        let mut writer = EmitterConfig::new()
            .write_document_declaration(false)
            .create_writer(dst);

        if let Err(e) = assemble_xml_tv(self, &mut writer) {
            Err(format!("{}", e))
        } else {
            Ok(())
        }
    }
}
