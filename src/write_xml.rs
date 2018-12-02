use std::io;
use std::collections::HashMap;

use chrono::{TimeZone, Utc};

use xml::common::XmlVersion;
use xml::writer::{Result, EventWriter, XmlEvent};

use epg::{Epg, FMT_DATETIME};
use mpegts::textcode;

type XmlResult = Result<()>;

fn write_xml_value<W: io::Write>(map: &HashMap<String, String>, w: &mut EventWriter<W>, name: &str) -> XmlResult {
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

fn write_xml_channel<W: io::Write>(epg: &Epg, w: &mut EventWriter<W>) -> XmlResult {
    for (id, channel) in &epg.channels {
        w.write(XmlEvent::start_element("channel").attr("id", id))?;

        write_xml_value(&channel.name, w, "display-name")?;

        w.write(XmlEvent::end_element())?;
        w.write(XmlEvent::Characters("\n"))?;
    }

    Ok(())
}

fn write_xml_programme<W: io::Write>(epg: &Epg, w: &mut EventWriter<W>) -> XmlResult {
    for (id, channel) in &epg.channels {
        for event in &channel.events {
            w.write(XmlEvent::start_element("programme")
                .attr("event_id", &event.event_id.to_string())
                .attr("channel", id)
                .attr("start", &Utc.timestamp(event.start, 0).format(FMT_DATETIME).to_string())
                .attr("stop", &Utc.timestamp(event.stop, 0).format(FMT_DATETIME).to_string()))?;

            write_xml_value(&event.title, w, "title")?;
            write_xml_value(&event.subtitle, w, "sub-title")?;
            write_xml_value(&event.desc, w, "desc")?;

            w.write(XmlEvent::end_element())?;
            w.write(XmlEvent::Characters("\n"))?;
        }
    }

    Ok(())
}

pub fn write_xml_tv<W: io::Write>(epg: &Epg, w: &mut EventWriter<W>) -> XmlResult {
    w.write(XmlEvent::StartDocument {
        version: XmlVersion::Version10,
        encoding: Some("utf-8"),
        standalone: None,
    })?;
    w.write(XmlEvent::Characters("\n"))?;

    w.write(XmlEvent::start_element("tv")
        .attr("generator-info-name", "Cesbo Astra")
        .attr("generator-info-url", "https://cesbo.com"))?;
    w.write(XmlEvent::Characters("\n"))?;

    write_xml_channel(epg, w)?;
    write_xml_programme(epg, w)?;

    w.write(XmlEvent::end_element())?;
    Ok(())
}
