use std::io;
use std::collections::HashMap;

use mpegts::psi::*;
use chrono::prelude::*;
use xml::attribute::OwnedAttribute;
use xml::common::XmlVersion;

use xml::reader::{ParserConfig, Events};
use xml::reader::Result as XmlReaderResultBase;
use xml::reader::XmlEvent as XmlReaderEvent;

use xml::writer::{EmitterConfig, EventWriter};
use xml::writer::Result as XmlWriterResultBase;
use xml::writer::XmlEvent as XmlWriterEvent;

type XmlReaderResult = XmlReaderResultBase<()>;
type XmlWriterResult = XmlWriterResultBase<()>;

#[derive(Default, Debug, Clone)]
pub struct EpgEvent {
    pub start: i64,
    pub stop: i64,
    pub title: HashMap<String, String>,
    pub subtitle: HashMap<String, String>,
    pub desc: HashMap<String, String>,
}

const FMT_DATETIME: &str = "%Y%m%d%H%M%S %z";

#[inline]
fn parse_date(value: &str) -> i64 {
    match DateTime::parse_from_str(value, FMT_DATETIME) {
        Ok(v) => v.timestamp(),
        Err(_) => 0,
    }
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

fn skip_xml_element<R: io::Read>(reader: &mut Events<R>) -> XmlReaderResult {
    let mut deep = 0;

    while let Some(e) = reader.next() {
        match e? {
            XmlReaderEvent::StartElement { .. } => deep += 1,
            XmlReaderEvent::EndElement { .. } if deep > 0 => deep -= 1,
            XmlReaderEvent::EndElement { .. } => return Ok(()),
            _ => {},
        };
    }

    unreachable!();
}

fn parse_xml_channel<R: io::Read>(epg: &mut Epg, reader: &mut Events<R>, attrs: &Vec<OwnedAttribute>) -> XmlReaderResult {
    let mut id = String::new();
    let mut event_id: usize = 0;

    for attr in attrs.iter() {
        match attr.name.local_name.as_str() {
            "id" => id.push_str(&attr.value),
            "event_id" => event_id = usize::from_str_radix(&attr.value, 10).unwrap_or(0),
            _ => {},
        };
    }

    if id.is_empty() {
        return skip_xml_element(reader);
    }

    let channel = epg.channels
        .entry(id)
        .or_insert(EpgChannel::default());
    channel.event_id = event_id;

    while let Some(e) = reader.next() {
        match e? {
            XmlReaderEvent::StartElement { .. } => skip_xml_element(reader)?,
            XmlReaderEvent::EndElement { .. } => return Ok(()),
            _ => {},
        };
    }

    unreachable!();
}

fn parse_xml_programme_info<R: io::Read>(info: &mut HashMap<String, String>, reader: &mut Events<R>, attrs: &Vec<OwnedAttribute>) -> XmlReaderResult {
    let mut lang = String::new();

    for attr in attrs.iter() {
        match attr.name.local_name.as_str() {
            "lang" => lang.push_str(&attr.value),
            _ => {},
        };
    }

    let value = info
        .entry(lang)
        .or_insert_with(|| String::new());

    while let Some(e) = reader.next() {
        match e? {
            XmlReaderEvent::StartElement { .. } => skip_xml_element(reader)?,
            XmlReaderEvent::EndElement { .. } => return Ok(()),
            XmlReaderEvent::Characters(v) => value.push_str(&v),
            _ => {},
        };
    }

    unreachable!();
}

fn parse_xml_programme<R: io::Read>(epg: &mut Epg, reader: &mut Events<R>, attrs: &Vec<OwnedAttribute>) -> XmlReaderResult {
    let mut id = String::new();
    let mut start: i64 = 0;
    let mut stop: i64 = 0;

    for attr in attrs.iter() {
        match attr.name.local_name.as_str() {
            "channel" => id.push_str(&attr.value),
            "start" => start = parse_date(&attr.value),
            "stop" => stop = parse_date(&attr.value),
            _ => {},
        };
    }

    let channel = match epg.channels.get_mut(&id) {
        Some(v) => v,
        None => return skip_xml_element(reader),
    };

    let mut event = EpgEvent::default();
    event.start = start;
    event.stop = stop;

    while let Some(e) = reader.next() {
        match e? {
            XmlReaderEvent::StartElement { name, attributes, .. } => match name.local_name.as_str() {
                "title" => parse_xml_programme_info(&mut event.title, reader, &attributes)?,
                "sub-title" => parse_xml_programme_info(&mut event.subtitle, reader, &attributes)?,
                "desc" => parse_xml_programme_info(&mut event.desc, reader, &attributes)?,
                _ => skip_xml_element(reader)?,
            },
            XmlReaderEvent::EndElement { .. } => {
                channel.events.push(event);
                return Ok(());
            },
            _ => {},
        };
    }

    unreachable!();
}

fn parse_xml_tv<R: io::Read>(epg: &mut Epg, reader: &mut Events<R>) -> XmlReaderResult {
    while let Some(e) = reader.next() {
        match e? {
            XmlReaderEvent::StartElement { name, attributes, .. } => {
                match name.local_name.as_str() {
                    "tv" => {},
                    "channel" => parse_xml_channel(epg, reader, &attributes)?,
                    "programme" => parse_xml_programme(epg, reader, &attributes)?,
                    _ => skip_xml_element(reader)?,
                };
            },
            XmlReaderEvent::EndDocument => return Ok(()),
            _ => {},
        };
    }

    unreachable!();
}

fn assemble_xml_channel<W: io::Write>(epg: &Epg, w: &mut EventWriter<W>) -> XmlWriterResult {
    for (id, channel) in epg.channels.iter() {
        w.write(XmlWriterEvent::start_element("channel")
            .attr("id", id)
            .attr("event_id", &channel.event_id.to_string()))?;
        // TODO: channel names
        w.write(XmlWriterEvent::start_element("display-name")
            .attr("lang", "en"))?;
        w.write(XmlWriterEvent::Characters("TODO"))?;
        w.write(XmlWriterEvent::end_element())?;
        w.write(XmlWriterEvent::end_element())?;
        w.write(XmlWriterEvent::Characters("\n"))?;
    }

    Ok(())
}

fn assemble_xml_programme_info<W: io::Write>(info: &HashMap<String, String>, w: &mut EventWriter<W>, name: &str) -> XmlWriterResult {
    for (lang, text) in info.iter() {
        w.write(XmlWriterEvent::start_element(name)
            .attr("lang", lang))?;
        w.write(XmlWriterEvent::Characters(text))?;
        w.write(XmlWriterEvent::end_element())?;
    }

    Ok(())
}

fn assemble_xml_programme<W: io::Write>(epg: &Epg, w: &mut EventWriter<W>) -> XmlWriterResult {
    for (id, channel) in epg.channels.iter() {
        for event in channel.events.iter() {
            // TODO: fix timezone
            w.write(XmlWriterEvent::start_element("programme")
                .attr("channel", id)
                .attr("start", &Local.timestamp(event.start, 0).format(FMT_DATETIME).to_string())
                .attr("stop", &Local.timestamp(event.stop, 0).format(FMT_DATETIME).to_string()))?;

            assemble_xml_programme_info(&event.title, w, "title")?;
            assemble_xml_programme_info(&event.subtitle, w, "sub-title")?;
            assemble_xml_programme_info(&event.desc, w, "desc")?;

            w.write(XmlWriterEvent::end_element())?;
            w.write(XmlWriterEvent::Characters("\n"))?;
        }
    }

    Ok(())
}

fn assemble_xml_tv<W: io::Write>(epg: &Epg, w: &mut EventWriter<W>) -> XmlWriterResult {
    w.write(XmlWriterEvent::StartDocument {
        version: XmlVersion::Version10,
        encoding: Some("utf-8"),
        standalone: None,
    })?;
    w.write(XmlWriterEvent::Characters("\n"))?;
    w.write(XmlWriterEvent::start_element("tv")
        .attr("generator-info-name", "Cesbo Astra")
        .attr("generator-info-url", "https://cesbo.com"))?;
    w.write(XmlWriterEvent::Characters("\n"))?;

    assemble_xml_channel(epg, w)?;
    assemble_xml_programme(epg, w)?;

    w.write(XmlWriterEvent::end_element())?;
    Ok(())
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
