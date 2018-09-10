extern crate epg;
use epg::*;

use std::str;

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
    println!("{}", xml);
}
