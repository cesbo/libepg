use std::io;
use std::collections::HashMap;

use mpegts::psi::*;
use chrono::prelude::*;

use xml::reader::ParserConfig;
use xml::writer::EmitterConfig;

use reader::parse_xml_tv;
use writer::assemble_xml_tv;

pub const FMT_DATETIME: &str = "%Y%m%d%H%M%S %z";

#[derive(Default, Debug, Clone)]
pub struct EpgEvent {
    pub start: i64,
    pub stop: i64,
    pub title: HashMap<String, String>,
    pub subtitle: HashMap<String, String>,
    pub desc: HashMap<String, String>,
}

impl EpgEvent {
    pub fn parse_eit(eit_item: &EitItem) -> EpgEvent {
        let mut event = EpgEvent::default();

        event.start = eit_item.start;
        event.stop = eit_item.start + eit_item.duration as i64;

        for desc in eit_item.descriptors.iter() {
            match desc {
                Descriptor::Desc4D(v) => {
                    if v.name.len() > 0 {
                        event.title
                            .entry(v.lang.to_string())
                            .or_insert_with(|| String::new())
                            .push_str(v.name.as_str());
                    }

                    if v.text.len() > 0 {
                        event.subtitle
                            .entry(v.lang.to_string())
                            .or_insert_with(|| String::new())
                            .push_str(v.text.as_str());
                    }
                },
                Descriptor::Desc4E(v) => {
                    if v.text.len() > 0 {
                        event.desc
                            .entry(v.lang.to_string())
                            .or_insert_with(|| String::new())
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

        for (lang, title) in self.title.iter() {
            let subtitle = match self.subtitle.get(lang) {
                Some(v) => v,
                None => "",
            };

            eit_item.descriptors.push(Descriptor::Desc4D(Desc4D {
                lang: lang.to_string(),
                name: title.to_string(),
                text: subtitle.to_string(),
                codepage: codepage,
            }));
        }

        for (lang, desc) in self.desc.iter() {
            eit_item.descriptors.push(Descriptor::Desc4E(Desc4E {
                number: 0,
                last_number: 0,
                lang: lang.to_string(),
                items: Vec::new(),
                text: desc.to_string(),
                codepage: codepage,
            }));
        }

        eit_item
    }
}

#[derive(Default, Debug)]
pub struct EpgChannel {
    pub event_id: usize,
    pub events: Vec<EpgEvent>,
}

impl EpgChannel {
    pub fn parse_eit(&mut self, eit: &Eit) {
        for eit_item in eit.items.iter() {
            self.events.push(EpgEvent::parse_eit(eit_item));
        }

        self.sort();
    }

    pub fn sort(&mut self) {
        self.events.sort_by(|a, b| a.start.cmp(&b.start));
    }

    pub fn assemble_eit(&self, codepage: usize) -> Eit {
        let mut eit = Eit::default();
        eit.table_id = 0x50;

        let current_time = Utc::now().timestamp();

        for event in self.events.iter() {
            let mut eit_item = event.assemble_eit(codepage);
            eit_item.event_id = (self.event_id as usize + eit.items.len()) as u16;
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
            for (_, channel) in self.channels.iter_mut() {
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
