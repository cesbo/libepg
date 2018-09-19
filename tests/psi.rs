extern crate epg;
extern crate mpegts;

use mpegts::psi::*;g
use mpegts::textcode;
use epg::*;

use std::str;

pub const EIT_50: &[u8] = &[
    0x47, 0x40, 0x12, 0x14, 0x00, 0x50, 0xf2, 0x21, 0x1c, 0xcf, 0xeb, 0x00, 0x00, 0x1c, 0xe8, 0x00,
    0x01, 0x00, 0x50, 0x7c, 0xcc, 0xe3, 0xe7, 0x18, 0x10, 0x00, 0x00, 0x30, 0x00, 0x12, 0x06, 0x4d,
    0x2c, 0x70, 0x6f, 0x6c, 0x27, 0x10, 0x00, 0x02, 0x4f, 0x73, 0x74, 0x61, 0x74, 0x6e, 0x69, 0x20,
    0x70, 0x72, 0x61, 0x77, 0x64, 0x7a, 0x69, 0x77, 0x79, 0x20, 0x6d, 0xea, 0xbf, 0x63, 0x7a, 0x79,
    0x7a, 0x6e, 0x61, 0x20, 0x34, 0x3a, 0x20, 0x6f, 0x64, 0x63, 0x2e, 0x35, 0x00, 0x4e, 0xfe, 0x01,
    0x70, 0x6f, 0x6c, 0x00, 0xf8, 0x10, 0x00, 0x02, 0x73, 0x65, 0x72, 0x69, 0x61, 0x6c, 0x20, 0x6b,
    0x6f, 0x6d, 0x65, 0x64, 0x69, 0x6f, 0x77, 0x79, 0x20, 0x28, 0x55, 0x53, 0x41, 0x2c, 0x20, 0x32,
    0x30, 0x31, 0x34, 0x29, 0x20, 0x6f, 0x64, 0x63, 0x2e, 0x35, 0x2c, 0x20, 0x53, 0x7a, 0x6b, 0x6f,
    0x6c, 0x6e, 0x61, 0x20, 0x66, 0x75, 0x7a, 0x6a, 0x61, 0x3f, 0x57, 0x79, 0x73, 0x74, 0xea, 0x70,
    0x75, 0x6a, 0xb1, 0x3a, 0x20, 0x54, 0x69, 0x6d, 0x20, 0x41, 0x6c, 0x6c, 0x65, 0x6e, 0x2c, 0x20,
    0x4e, 0x61, 0x6e, 0x63, 0x79, 0x20, 0x54, 0x72, 0x61, 0x76, 0x69, 0x73, 0x2c, 0x20, 0x4d, 0x6f,
    0x6c, 0x6c, 0x79, 0x20, 0x45, 0x70, 0x68, 0x72, 0x61, 0x69, 0x6d, 0x3f,
    0x47, 0x00, 0x12, 0x15, 0x4d, 0x69, 0x6b, 0x65, 0x20, 0x69, 0x20, 0x43, 0x68, 0x75, 0x63, 0x6b,
    0x20, 0x64, 0x65, 0x62, 0x61, 0x74, 0x75, 0x6a, 0xb1, 0x20, 0x6e, 0x61, 0x20, 0x74, 0x65, 0x6d,
    0x61, 0x74, 0x20, 0x7a, 0x61, 0x6c, 0x65, 0x74, 0x20, 0x6c, 0x6f, 0x6b, 0x61, 0x6c, 0x6e, 0x65,
    0x67, 0x6f, 0x20, 0x72, 0x65, 0x66, 0x65, 0x72, 0x65, 0x6e, 0x64, 0x75, 0x6d, 0x20, 0x6f, 0x20,
    0x70, 0x6f, 0xb3, 0xb1, 0x63, 0x7a, 0x65, 0x6e, 0x69, 0x75, 0x20, 0x69, 0x63, 0x68, 0x20, 0x65,
    0x6b, 0x73, 0x6b, 0x6c, 0x75, 0x7a, 0x79, 0x77, 0x6e, 0x65, 0x6a, 0x20, 0x73, 0x7a, 0x6b, 0x6f,
    0xb3, 0x79, 0x20, 0xb6, 0x72, 0x65, 0x64, 0x6e, 0x69, 0x65, 0x6a, 0x20, 0x7a, 0x20, 0x73, 0xb1,
    0x73, 0x69, 0x65, 0x64, 0x7a, 0x74, 0x77, 0x61, 0x20, 0x7a, 0x20, 0x70, 0x6c, 0x61, 0x63, 0xf3,
    0x77, 0x6b, 0xb1, 0x20, 0x7a, 0x65, 0x20, 0xb6, 0x72, 0xf3, 0x64, 0x6d, 0x69, 0x65, 0xb6, 0x63,
    0x69, 0x61, 0x2e, 0x20, 0x5a, 0x4e, 0xd0, 0x11, 0x70, 0x6f, 0x6c, 0x00, 0xca, 0x10, 0x00, 0x02,
    0x20, 0x6f, 0x6b, 0x61, 0x7a, 0x6a, 0x69, 0x20, 0x48, 0x61, 0x6c, 0x6c, 0x6f, 0x77, 0x65, 0x65,
    0x6e, 0x2c, 0x20, 0x52, 0x79, 0x61, 0x6e, 0x20, 0x70, 0x72, 0x7a, 0x65,
    0x47, 0x00, 0x12, 0x16, 0x62, 0x69, 0x65, 0x72, 0x61, 0x20, 0x42, 0x6f, 0x79, 0x64, 0x61, 0x20,
    0x7a, 0x61, 0x20, 0x62, 0x72, 0x79, 0xb3, 0xea, 0x20, 0x77, 0xea, 0x67, 0x6c, 0x61, 0x2e, 0x20,
    0x4d, 0x61, 0x20, 0x74, 0x6f, 0x20, 0x62, 0x79, 0xe6, 0x20, 0x6b, 0x6f, 0x6c, 0x65, 0x6a, 0x6e,
    0x79, 0x6d, 0x20, 0x70, 0x72, 0x7a, 0x79, 0x70, 0x6f, 0x6d, 0x6e, 0x69, 0x65, 0x6e, 0x69, 0x65,
    0x6d, 0x20, 0x64, 0x6c, 0x61, 0x20, 0x56, 0x61, 0x6e, 0x65, 0x73, 0x73, 0x79, 0x2c, 0x20, 0xbf,
    0x65, 0x20, 0x6a, 0x65, 0x6a, 0x20, 0x70, 0x72, 0x61, 0x63, 0x61, 0x20, 0x6a, 0x61, 0x6b, 0x6f,
    0x20, 0x67, 0x65, 0x6f, 0x6c, 0x6f, 0x67, 0x61, 0x20, 0x6d, 0x6f, 0xbf, 0x65, 0x20, 0x73, 0x7a,
    0x6b, 0x6f, 0x64, 0x7a, 0x69, 0xe6, 0x20, 0xb6, 0x72, 0x6f, 0x64, 0x6f, 0x77, 0x69, 0x73, 0x6b,
    0x75, 0x20, 0x6e, 0x61, 0x74, 0x75, 0x72, 0x61, 0x6c, 0x6e, 0x65, 0x6d, 0x75, 0x2e, 0x3f, 0x52,
    0x65, 0xbf, 0x79, 0x73, 0x65, 0x72, 0x3a, 0x20, 0x4a, 0x6f, 0x68, 0x6e, 0x20, 0x50, 0x61, 0x73,
    0x71, 0x75, 0x69, 0x6e, 0x3f, 0x4f, 0x64, 0x20, 0x6c, 0x61, 0x74, 0x3a, 0x20, 0x31, 0x32, 0x55,
    0x04, 0x50, 0x4c, 0x20, 0x09, 0xff, 0xc2, 0xe2, 0x2a, 0xff, 0xff, 0xff,
];
pub const EIT_50_EVENT_TITLE: &str = "Ostatni prawdziwy mężczyzna 4: odc.5";
pub const EIT_50_EVENT_DESC: &str = "serial komediowy (USA, 2014) odc.5, Szkolna fuzja?Występują: Tim Allen, Nancy Travis, Molly Ephraim?Mike i Chuck debatują na temat zalet lokalnego referendum o połączeniu ich ekskluzywnej szkoły średniej z sąsiedztwa z placówką ze śródmieścia. Z okazji Halloween, Ryan przebiera Boyda za bryłę węgla. Ma to być kolejnym przypomnieniem dla Vanessy, że jej praca jako geologa może szkodzić środowisku naturalnemu.?Reżyser: John Pasquin?Od lat: 12";

#[test]
fn test_parse_eit() {
    let mut psi = Psi::default();

    let mut skip = 0;
    while skip < EIT_50.len() {
        psi.mux(&EIT_50[skip ..]);
        skip += 188;
    }
    assert!(psi.check());

    let mut eit = Eit::default();
    eit.parse(&psi);

    let mut channel = EpgChannel::default();
    channel.parse_eit(&eit);

    assert_eq!(channel.name.len(), 0);

    assert_eq!(channel.events.len(), 1);
    let event = channel.events.iter().next().unwrap();
    assert_eq!(event.start, 1534183800);
    assert_eq!(event.stop, 1534183800 + 1800);
    assert_eq!(event.title.len(), 1);
    assert_eq!(event.title.get("pol").unwrap(), EIT_50_EVENT_TITLE);
    assert_eq!(event.subtitle.len(), 0);
    assert_eq!(event.desc.len(), 1);
    assert_eq!(event.desc.get("pol").unwrap(), EIT_50_EVENT_DESC);

    assert_eq!(channel.last_event_start, event.start);
}

#[test]
fn test_ts_to_xmltv() {
    let mut psi = Psi::default();

    let mut skip = 0;
    while skip < EIT_50.len() {
        psi.mux(&EIT_50[skip ..]);
        skip += 188;
    }
    assert!(psi.check());

    let mut eit = Eit::default();
    eit.parse(&psi);

    let mut channel = EpgChannel::default();
    channel.parse_eit(&eit);
    channel.name.insert("pl".to_string(), "Test".to_string());

    let mut epg = Epg::default();
    epg.channels.insert("id-1".to_string(), channel);

    let mut dst: Vec<u8> = Vec::new();
    epg.assemble_xml(&mut dst).unwrap();

    let xml = str::from_utf8(&dst).unwrap();
    println!("{}", xml);

    // TODO: more tests
}

#[test]
fn test_assemble_eit() {
    let mut psi = Psi::default();

    let mut skip = 0;
    while skip < EIT_50.len() {
        psi.mux(&EIT_50[skip ..]);
        skip += 188;
    }
    assert!(psi.check());

    let mut eit = Eit::default();
    eit.parse(&psi);

    let mut channel = EpgChannel::default();
    channel.parse_eit(&eit);

    channel.first_event_id = 1;
    let mut tmp_eit = channel.assemble_eit(textcode::ISO8859_2);
    tmp_eit.version = 1;
    tmp_eit.pnr = 6;
    tmp_eit.tsid = 1;
    tmp_eit.onid = 1;

    let mut new_psi = Psi::default();
    tmp_eit.assemble(&mut new_psi);

    let mut new_eit = Eit::default();
    new_eit.parse(&new_psi);

    let mut new_channel = EpgChannel::default();
    new_channel.parse_eit(&new_eit);

    let event = channel.events.iter().next().unwrap();
    let new_event = new_channel.events.iter().next().unwrap();

    assert_eq!(event, new_event);
}
