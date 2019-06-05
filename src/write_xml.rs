use std::{
    io,
    collections::HashMap,
};

use chrono::{
    TimeZone,
    Utc,
};

use xml::{
    common::XmlVersion,
    writer::{
        self,
        EventWriter,
        XmlEvent,
        EmitterConfig,
    },
};

use mpegts::textcode;

use crate::{
    Epg,
    FMT_DATETIME,
};


#[derive(Debug, Error)]
pub enum XmlWriterError {
    #[error_from("XmlWriter: {}", 0)]
    XmlWriter(writer::Error),
}


type Result<T> = std::result::Result<T, XmlWriterError>;


fn write_xml_value<W: io::Write>(
    map: &HashMap<String, String>,
    w: &mut EventWriter<W>,
    name: &str) -> Result<()>
{
    for (lang, text) in map.iter() {
        let lang = match textcode::lang::convert(lang) {
            Some(v) => v,
            None => continue,
        };

        w.write(XmlEvent::start_element(name).attr("lang", lang))?;
        w.write(XmlEvent::Characters(text))?;
        w.write(XmlEvent::end_element())?;
    }

    Ok(())
}


fn write_xml_channel<W: io::Write>(
    epg: &Epg,
    w: &mut EventWriter<W>) -> Result<()>
{
    for (id, channel) in &epg.channels {
        w.write(XmlEvent::start_element("channel").attr("id", id))?;

        write_xml_value(&channel.name, w, "display-name")?;

        w.write(XmlEvent::end_element())?;
        w.write(XmlEvent::Characters("\n"))?;
    }

    Ok(())
}


fn write_xml_programme<W: io::Write>(
    epg: &Epg,
    w: &mut EventWriter<W>) -> Result<()>
{
    for (id, channel) in &epg.channels {
        for event in &channel.events {
            w.write(XmlEvent::start_element("programme")
                .attr("event_id", &event.event_id.to_string())
                .attr("channel", id)
                .attr("start", &Utc.timestamp(event.start as i64, 0).format(FMT_DATETIME).to_string())
                .attr("stop", &Utc.timestamp(event.stop as i64, 0).format(FMT_DATETIME).to_string()))?;

            write_xml_value(&event.title, w, "title")?;
            write_xml_value(&event.subtitle, w, "sub-title")?;
            write_xml_value(&event.desc, w, "desc")?;

            w.write(XmlEvent::end_element())?;
            w.write(XmlEvent::Characters("\n"))?;
        }
    }

    Ok(())
}


pub fn write_xml_tv<W: io::Write>(
    epg: &Epg,
    dst: W) -> Result<()>
{
    let mut writer = EmitterConfig::new()
        .write_document_declaration(false)
        .create_writer(dst);

    writer.write(XmlEvent::StartDocument {
        version: XmlVersion::Version10,
        encoding: Some("utf-8"),
        standalone: None,
    })?;
    writer.write(XmlEvent::Characters("\n"))?;

    writer.write(XmlEvent::start_element("tv")
        .attr("generator-info-name", "Cesbo Astra")
        .attr("generator-info-url", "https://cesbo.com"))?;
    writer.write(XmlEvent::Characters("\n"))?;

    write_xml_channel(epg, &mut writer)?;
    write_xml_programme(epg, &mut writer)?;

    writer.write(XmlEvent::end_element())?;

    Ok(())
}
