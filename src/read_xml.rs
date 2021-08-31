use std::{
    io,
    collections::HashMap,
};

use chrono::{
    DateTime,
    TimeZone,
    Utc,
};

use xml::{
    attribute::OwnedAttribute,
    reader::{
        self,
        Events,
        XmlEvent,
        ParserConfig,
    },
};

use mpegts::textcode;

use crate::{
    Epg,
    EpgChannel,
    EpgEvent,
    FMT_DATETIME,
};


#[derive(Debug, Error)]
pub enum XmlReaderError {
    #[error_from("XmlReader: {}", 0)]
    XmlReader(reader::Error),
}


type Result<T> = std::result::Result<T, XmlReaderError>;


fn parse_date(value: &str) -> u64 {
    if value.len() > 14 {
        match DateTime::parse_from_str(value, FMT_DATETIME) {
            Ok(v) => v.timestamp() as u64,
            _ => 0,
        }
    } else if (value.len() == 14) || (value.len() == 12) {
        /* 14: %Y%m%d%H%M%S */
        /* 12: %Y%m%d%H%M */
        let x = value.len() - 2;
        match Utc.datetime_from_str(value, &FMT_DATETIME[.. x]) {
            Ok(v) => v.timestamp() as u64,
            _ => 0,
        }
    } else {
        0
    }
}


fn skip_xml_element<R: io::Read>(reader: &mut Events<R>) -> Result<()> {
    let mut deep = 0;

    for e in reader {
        match e? {
            XmlEvent::StartElement { .. } => deep += 1,
            XmlEvent::EndElement { .. } if deep > 0 => deep -= 1,
            XmlEvent::EndElement { .. } => return Ok(()),
            _ => {},
        };
    }

    unreachable!();
}


fn parse_xml_value<R: io::Read>(
    map: &mut HashMap<String, String>,
    reader: &mut Events<R>,
    attrs: &[OwnedAttribute]) -> Result<()>
{
    let mut lang = String::new();

    for attr in attrs.iter() {
        if attr.name.local_name.as_str() == "lang" {
            if let Some(v) = textcode::lang::convert(&attr.value) {
                lang.push_str(v);
            };
        }
    }

    if lang.is_empty() {
        lang.push_str("und"); /* ISO 639-2 Undetermined */
    }

    let value = map
        .entry(lang)
        .or_insert_with(String::new);

    while let Some(e) = reader.next() {
        match e? {
            XmlEvent::StartElement { .. } => skip_xml_element(reader)?,
            XmlEvent::EndElement { .. } => return Ok(()),
            XmlEvent::Characters(v) => value.push_str(&v),
            _ => {},
        };
    }

    unreachable!();
}


fn read_xml_channel<R: io::Read>(
    epg: &mut Epg,
    reader: &mut Events<R>,
    attrs: &[OwnedAttribute]) -> Result<()>
{
    let mut id = String::new();

    for attr in attrs.iter() {
        if attr.name.local_name.as_str() == "id" {
            id.push_str(&attr.value);
        }
    }

    if id.is_empty() {
        return skip_xml_element(reader);
    }

    if epg.channels.contains_key(&id) {
        return skip_xml_element(reader);
    }

    let mut channel = EpgChannel::default();
    while let Some(e) = reader.next() {
        match e? {
            XmlEvent::StartElement { name, attributes, .. } => match name.local_name.as_str() {
                "display-name" => parse_xml_value(&mut channel.name, reader, &attributes)?,
                _ => skip_xml_element(reader)?,
            },
            XmlEvent::EndElement { .. } => {
                epg.channels.insert(id, channel);
                return Ok(());
            },
            _ => {},
        };
    }

    unreachable!();
}


fn read_xml_programme<R: io::Read>(
    epg: &mut Epg,
    reader: &mut Events<R>,
    attrs: &[OwnedAttribute]) -> Result<()>
{
    let mut event_id: u16 = 0;
    let mut channel = String::new();
    let mut start: u64 = 0;
    let mut stop: u64 = 0;

    for attr in attrs.iter() {
        match attr.name.local_name.as_str() {
            "event_id" => event_id = attr.value.parse::<u16>().unwrap_or(0),
            "channel" => channel.push_str(&attr.value),
            "start" => start = parse_date(&attr.value),
            "stop" => stop = parse_date(&attr.value),
            _ => {},
        };
    }

    let channel = match epg.channels.get_mut(&channel) {
        Some(v) => v,
        None => return skip_xml_element(reader),
    };

    if channel.last_event_start >= start {
        return skip_xml_element(reader);
    }

    let mut event = EpgEvent {
        event_id,
        start,
        stop,
        ..Default::default()
    };

    while let Some(e) = reader.next() {
        match e? {
            XmlEvent::StartElement { name, attributes, .. } => match name.local_name.as_str() {
                "title" => parse_xml_value(&mut event.title, reader, &attributes)?,
                "sub-title" => parse_xml_value(&mut event.subtitle, reader, &attributes)?,
                "desc" => parse_xml_value(&mut event.desc, reader, &attributes)?,
                _ => skip_xml_element(reader)?,
            },
            XmlEvent::EndElement { .. } => {
                channel.events.push(event);
                return Ok(());
            },
            _ => {},
        };
    }

    unreachable!();
}


pub fn read_xml_tv<R: io::Read>(
    epg: &mut Epg,
    src: R) -> Result<()>
{
    let mut reader = ParserConfig::new()
        .trim_whitespace(true)
        .ignore_comments(true)
        .create_reader(src)
        .into_iter();

    while let Some(e) = reader.next() {
        match e? {
            XmlEvent::StartElement { name, attributes, .. } => {
                match name.local_name.as_str() {
                    "tv" => {},
                    "channel" => read_xml_channel(epg, &mut reader, &attributes)?,
                    "programme" => read_xml_programme(epg, &mut reader, &attributes)?,
                    _ => skip_xml_element(&mut reader)?,
                };
            }
            XmlEvent::EndDocument => {
                for channel in epg.channels.values_mut() {
                    channel.sort();
                }
                return Ok(());
            }
            _ => {}
        };
    }

    unreachable!();
}
