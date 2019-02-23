use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::collections::HashMap;

use crate::error::{Error, Result};

use mpegts::psi::*;
use mpegts::textcode::*;
use chrono::Utc;

use xml::reader::ParserConfig;
use xml::writer::EmitterConfig;

use crate::read_xml::read_xml_tv;
use crate::write_xml::write_xml_tv;

use curl;

pub const FMT_DATETIME: &str = "%Y%m%d%H%M%S %z";

// TODO: HashMap for codepage: language = codepage

#[derive(Default, Debug, Clone, PartialEq)]
pub struct EpgEvent {
    /// Unique event identifier
    pub event_id: u16,
    /// Event start time
    pub start: u64,
    /// Event stop tiem (equal to the next event start time)
    pub stop: u64,
    /// Event title list
    pub title: HashMap<String, String>,
    /// Event short description list
    pub subtitle: HashMap<String, String>,
    /// Event description list
    pub desc: HashMap<String, String>,
    /// Codepage
    pub codepage: u8,
}

impl<'a> From<&'a EitItem> for EpgEvent {
    fn from(eit_item: &EitItem) -> Self {
        let mut event = EpgEvent::default();

        event.event_id = eit_item.event_id;
        event.start = eit_item.start;
        event.stop = eit_item.start + u64::from(eit_item.duration);

        for desc in eit_item.descriptors.iter() {
            match desc {
                Descriptor::Desc4D(v) => {
                    event.title.insert(v.lang.to_string(), v.name.to_string());

                    if !v.text.is_empty() {
                        event.subtitle
                            .entry(v.lang.to_string())
                            .or_insert_with(String::new)
                            .push_str(&v.text.to_string());
                    }
                },
                Descriptor::Desc4E(v) => {
                    if !v.text.is_empty() {
                        event.desc
                            .entry(v.lang.to_string())
                            .or_insert_with(String::new)
                            .push_str(&v.text.to_string());
                    }
                },
                _ => (),
            };
        }

        event
    }
}

impl<'a> From<&'a EpgEvent> for EitItem {
    fn from(event: &EpgEvent) -> Self {
        let mut eit_item = EitItem::default();

        eit_item.event_id = event.event_id;
        eit_item.start = event.start;
        eit_item.duration = (event.stop - event.start) as u32;

        let current_time = Utc::now().timestamp() as u64;
        if current_time >= event.start && current_time < event.stop {
            eit_item.status = 4;
        } else {
            eit_item.status = 1;
        }

        for (lang, title) in &event.title {
            let subtitle = match event.subtitle.get(lang) {
                Some(v) => v,
                None => "",
            };

            eit_item.descriptors.push(Descriptor::Desc4D(Desc4D {
                lang: StringDVB::from_str(lang, 0),
                name: StringDVB::from_str(title, event.codepage),
                text: StringDVB::from_str(subtitle, event.codepage),
            }));
        }

        for (lang, desc) in &event.desc {
            let mut text_list = StringDVB::from_str(desc, event.codepage);
            text_list.truncate(1000);
            let mut text_list = text_list.split(0xFF - Desc4E::min_size());
            let mut number: u8 = 0;
            let last_number: u8 = text_list.len() as u8 - 1;

            while ! text_list.is_empty() {
                let text = text_list.remove(0);
                eit_item.descriptors.push(Descriptor::Desc4E(Desc4E {
                    number,
                    last_number,
                    lang: StringDVB::from_str(lang, 0),
                    items: Vec::new(),
                    text,
                }));
                number += 1;
            }
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
    /// Start time for last event
    pub last_event_start: u64,
}

impl EpgChannel {
    pub fn parse(&mut self, eit: &Eit) {
        for eit_item in &eit.items {
            self.events.push(EpgEvent::from(eit_item));
        }
        self.sort();
    }

    pub fn sort(&mut self) {
        if self.events.is_empty() {
            return;
        }

        self.events.sort_by(|a, b| a.start.cmp(&b.start));

        self.last_event_start = self.events.last().unwrap().start;

        let mut event_id = self.events.first().unwrap().event_id;
        for event in &mut self.events {
            event.event_id = event_id;
            event_id += 1;
        }
    }
}

#[derive(Default, Debug)]
pub struct Epg {
    pub channels: HashMap<String, EpgChannel>,
}

impl Epg {
    pub fn load(&mut self, src: &str) -> Result<()> {
        let url = src.splitn(2, "://").collect::<Vec<&str>>();

        if url.len() == 1 {
            let fh = File::open(url[0])?;
            return self.read(BufReader::new(fh));
        }

        match url[0] {
            "file" => {
                let fh = File::open(url[1])?;
                return self.read(BufReader::new(fh));
            },
            "http" | "https" => {
                let mut body = Vec::new();

                let mut request = curl::easy::Easy::new();
                request.url(src)?;

                {
                    let mut transfer = request.transfer();
                    transfer.write_function(
                        |data| {
                            body.extend_from_slice(data);
                            Ok(data.len())
                        }
                    )?;
                    transfer.perform()?;
                }

                return self.read(body.as_slice());
            }
            _ => return Err(Error::from(format!("unknown source type: {}", url[0]))),
        };

    }

    pub fn read<R: Read>(&mut self, src: R) -> Result<()> {
        let mut reader = ParserConfig::new()
            .trim_whitespace(true)
            .ignore_comments(true)
            .create_reader(src)
            .into_iter();

        read_xml_tv(self, &mut reader)?;

        for channel in self.channels.values_mut() {
            channel.sort();
        }
        Ok(())
    }

    pub fn write<W: Write>(&self, dst: W) -> Result<()> {
        let mut writer = EmitterConfig::new()
            .write_document_declaration(false)
            .create_writer(dst);

        write_xml_tv(self, &mut writer)?;

        Ok(())
    }
}
