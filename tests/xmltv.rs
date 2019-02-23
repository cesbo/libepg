extern crate epg;
extern crate mpegts;

use epg::*;

use mpegts::psi::*;
use mpegts::textcode::*;

use std::str;

#[test]
fn test_parse_programme() {
    let mut epg = Epg::default();
    epg.load("file://tests/docs/e1.xml").unwrap();
    let p = epg.channels.get("id-1").unwrap().events.get(0).unwrap();

    // check event
    assert_eq!(p.start, 1216103400);
    assert_eq!(p.stop, 1216060200);
    assert_eq!(p.title.get("eng").unwrap(), "Title");
    assert_eq!(p.desc.get("eng").unwrap(), "Desc");
}

#[test]
fn test_parse_xmltv() {
    let mut epg = Epg::default();

    for url in &["file://tests/docs/e2.xml", "https://pastebin.com/raw/dTk32xRZ"] {
        epg.load(url).unwrap();

        // get channel events
        let mut events_iter = epg.channels.get("id-1").unwrap().events.iter();
        // check event
        let p1 = events_iter.next().unwrap();
        assert_eq!(p1.title.get("eng").unwrap(), "Title #1");
        // check event
        let p2 = events_iter.next().unwrap();
        assert_eq!(p2.title.get("eng").unwrap(), "Title #2");
    }
}

#[test]
fn test_assemble_xmltv() {
    let mut epg = Epg::default();
    epg.load("file://tests/docs/e2.xml").unwrap();

    // convert epg into xmltv
    let mut target: Vec<u8> = Vec::new();
    epg.write(&mut target).unwrap();

    let xml = str::from_utf8(&target).unwrap();

    // TODO:
    println!("{}", xml);
}

#[test]
fn test_merge_xmltv() {
    let mut epg = Epg::default();
    epg.load("file://tests/docs/e3-1.xml").unwrap();

    epg.load("file://tests/docs/e3-2.xml").unwrap();

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
    let mut epg = Epg::default();
    epg.load("file://tests/docs/e4.xml").unwrap();

    let channel = epg.channels.get_mut("id-1").unwrap();
    let event = channel.events.iter_mut().next().unwrap();
    event.codepage = ISO8859_5;

    let mut eit = Eit::default();
    eit.table_id = 0x50;
    eit.version = 1;
    eit.pnr = 100;
    eit.tsid = 1;
    eit.onid = 1;
    eit.items.push(EitItem::from(&*event));

    // TODO: more tests
}
