extern crate epg;
extern crate mpegts;

use epg::*;

use mpegts::psi::*;
use mpegts::textcode::*;

use std::str;

#[test]
fn test_parse_programme() {
    let content: &[u8] = include_bytes!("docs/e1.xml");

    // convert xmltv into epg
    let mut epg = Epg::default();
    epg.parse_xml(content).unwrap();
    let p = epg.channels.get("id-1").unwrap().events.get(0).unwrap();

    // check event
    assert_eq!(p.start, 1216103400);
    assert_eq!(p.stop, 1216060200);
    assert_eq!(p.title.get("eng").unwrap(), "Title");
    assert_eq!(p.desc.get("eng").unwrap(), "Desc");
}

#[test]
fn test_parse_xmltv() {
    let content: &[u8] = include_bytes!("docs/e2.xml");

    // convert xmltv into epg
    let mut epg = Epg::default();
    epg.parse_xml(content).unwrap();

    // get channel events
    let mut events_iter = epg.channels.get("id-1").unwrap().events.iter();
    // check event
    let p1 = events_iter.next().unwrap();
    assert_eq!(p1.title.get("eng").unwrap(), "Title #1");
    // check event
    let p2 = events_iter.next().unwrap();
    assert_eq!(p2.title.get("eng").unwrap(), "Title #2");
}

#[test]
fn test_assemble_xmltv() {
    let content: &[u8] = include_bytes!("docs/e2.xml");

    // convert xmltv into epg
    let mut epg = Epg::default();
    epg.parse_xml(content).unwrap();

    // convert epg into xmltv
    let mut target: Vec<u8> = Vec::new();
    epg.assemble_xml(&mut target).unwrap();

    let xml = str::from_utf8(&target).unwrap();

    // TODO:
    println!("{}", xml);
}

#[test]
fn test_merge_xmltv() {
    let content: &[u8] = include_bytes!("docs/e3-1.xml");
    let mut epg = Epg::default();
    epg.parse_xml(content).unwrap();

    let content: &[u8] = include_bytes!("docs/e3-2.xml");
    epg.parse_xml(content).unwrap();

    let channel = epg.channels.get("id-1").unwrap();
    assert_eq!(channel.name.get("eng").unwrap(), "Test Channel");
    assert_eq!(channel.events.len(), 4);

    let ev0 = channel.events.get(0).unwrap();
    let ev1 = channel.events.get(1).unwrap();
    let ev2 = channel.events.get(2).unwrap();
    let ev3 = channel.events.get(3).unwrap();

    assert_eq!(ev0.stop, ev1.start);
    assert_eq!(ev1.stop, ev2.start);
    assert_eq!(ev2.stop, ev3.start);
}

#[test]
fn test_convert_to_psi() {
    let content: &[u8] = include_bytes!("docs/e4.xml");

    // convert xmltv into epg
    let mut epg = Epg::default();
    epg.parse_xml(content).unwrap();

    let channel = epg.channels.get("id-1").unwrap();
    let mut eit = channel.assemble(ISO8859_5);
    eit.version = 1;
    eit.pnr = 100;
    eit.tsid = 1;
    eit.onid = 1;

    let mut psi = Psi::default();
    eit.assemble(&mut psi);

    // TODO: more tests
}
