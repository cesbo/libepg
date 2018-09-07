extern crate epg;
use epg::*;

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
    assert_eq!(p.title.get("en").unwrap(), "Title");
    assert_eq!(p.desc.get("en").unwrap(), "Desc");
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
    assert_eq!(p1.title.get("en").unwrap(), "Title #1");
    // check event
    let p2 = events_iter.next().unwrap();
    assert_eq!(p2.title.get("en").unwrap(), "Title #2");
}
