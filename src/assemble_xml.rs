use std::io;
use std::collections::HashMap;

use chrono::prelude::*;

use xml::common::XmlVersion;
use xml::writer::{Result, EventWriter, XmlEvent};

use epg::{Epg, FMT_DATETIME};
use mpegts::textcode;

type XmlResult = Result<()>;

fn assemble_xml_value<W: io::Write>(map: &HashMap<String, String>, w: &mut EventWriter<W>, name: &str) -> XmlResult {
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

fn assemble_xml_channel<W: io::Write>(epg: &Epg, w: &mut EventWriter<W>) -> XmlResult {
    for (id, channel) in epg.channels.iter() {
        w.write(XmlEvent::start_element("channel")
            .attr("id", id))?;

        assemble_xml_value(&channel.name, w, "display-name")?;

        w.write(XmlEvent::end_element())?;
        w.write(XmlEvent::Characters("\n"))?;
    }

    Ok(())
}

fn assemble_xml_programme<W: io::Write>(epg: &Epg, w: &mut EventWriter<W>) -> XmlResult {
    for (id, channel) in epg.channels.iter() {
        for event in channel.events.iter() {
            w.write(XmlEvent::start_element("programme")
                .attr("channel", id)
                .attr("id", &event.event_id.to_string())
                .attr("start", &Local.timestamp(event.start, 0).format(FMT_DATETIME).to_string())
                .attr("stop", &Local.timestamp(event.stop, 0).format(FMT_DATETIME).to_string()))?;

            assemble_xml_value(&event.title, w, "title")?;
            assemble_xml_value(&event.subtitle, w, "sub-title")?;
            assemble_xml_value(&event.desc, w, "desc")?;

            w.write(XmlEvent::end_element())?;
            w.write(XmlEvent::Characters("\n"))?;
        }
    }

    Ok(())
}

pub fn assemble_xml_tv<W: io::Write>(epg: &Epg, w: &mut EventWriter<W>) -> XmlResult {
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

    assemble_xml_channel(epg, w)?;
    assemble_xml_programme(epg, w)?;

    w.write(XmlEvent::end_element())?;
    Ok(())
}
