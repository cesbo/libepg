extern crate epg;
extern crate xml;

use epg::*;

mod data;
use data::*;

#[test]
fn test_parse_programme() {
    let s = r#"<programme start="20080715003000 -0600" stop="20080715003000 +0600" channel="id-1">
<title lang="en">Title</title>
<desc lang="en">Desc</desc>
</programme>"#;
    let xml = xml::Node::from_str(s).unwrap();
    let xml = xml.iter_child().next().unwrap();
    let p = EpgEvent::parse_xml(&xml);
    assert_eq!(p.start, 1216103400);
    assert_eq!(p.stop, 1216060200);
    assert_eq!(p.title.get("en").unwrap(), "Title");
    assert_eq!(p.desc.get("en").unwrap(), "Desc");
}

#[test]
fn test_parse_xmltv() {
    // parse xml
    let xml = xml::Node::from_str(EXAMPLE).unwrap();
    let xml = xml.iter_child().next().unwrap();

    // convert xmltv into epg
    let mut epg = Epg::default();
    epg.parse_xml(&xml);
    // get channel events
    let mut events_iter = epg.channels.get("id-1").unwrap().events.iter();
    // check event
    let p1 = events_iter.next().unwrap();
    assert_eq!(p1.title.get("en").unwrap(), "Title #1");
    // check event
    let p2 = events_iter.next().unwrap();
    assert_eq!(p2.title.get("en").unwrap(), "Title #2");
}
