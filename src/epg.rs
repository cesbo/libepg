use std::collections::HashMap;

use xml;
use mpegts::psi::*;

#[derive(Default, Debug, Clone)]
pub struct EpgEvent {
    pub start: u64,
    pub stop: u64,
    pub title: HashMap<String, String>,
    pub subtitle: HashMap<String, String>,
    pub desc: HashMap<String, String>,
}

impl EpgEvent {
    pub fn parse_xml(&mut self, node: &xml::Node) {
        for i in node.iter_attr() {
            match i.key.as_str() {
                "start" => self.start = parse_date(&i.value),
                "stop" => self.stop = parse_date(&i.value),
                _ => (),
            };
        }

        for i in node.iter_child() {
            match i.key.as_str() {
                "title" => {
                    match i.get_attr("lang") {
                        Some(v) => self.title.insert(v.to_string(), i.text.clone()),
                        None => continue,
                    };
                },
                "sub-title" => {
                    match i.get_attr("lang") {
                        Some(v) => self.subtitle.insert(v.to_string(), i.text.clone()),
                        None => continue,
                    };
                },
                "desc" => {
                    match i.get_attr("lang") {
                        Some(v) => self.desc.insert(v.to_string(), i.text.clone()),
                        None => continue,
                    };
                },
                _ => (),
            };
        }
    }

    pub fn parse_eit(&mut self, eit_item: &EitItem) {
        self.start = eit_item.start;
        self.stop = eit_item.start + eit_item.duration as u64;

        for desc in eit_item.descriptors.iter() {
            match desc {
                Descriptor::Desc4D(v) => {
                    if v.name.len() > 0 {
                        self.title
                            .entry(v.lang.to_string())
                            .or_insert_with(|| String::new())
                            .push_str(v.name.as_str());
                    }

                    if v.text.len() > 0 {
                        self.subtitle
                            .entry(v.lang.to_string())
                            .or_insert_with(|| String::new())
                            .push_str(v.text.as_str());
                    }
                },
                Descriptor::Desc4E(v) => {
                    if v.text.len() > 0 {
                        self.desc
                            .entry(v.lang.to_string())
                            .or_insert_with(|| String::new())
                            .push_str(v.text.as_str());
                    }
                },
                _ => (),
            };
        }
    }
}

#[derive(Default, Debug)]
pub struct EpgChannel {
    pub pnr: u16,
    pub tsid: u16,
    pub onid: u16,
    pub events: Vec<EpgEvent>,
}

impl EpgChannel {
    pub fn parse_xml(&mut self, node: &xml::Node) {
        let mut event = EpgEvent::default();
        event.parse_xml(node);
        self.events.push(event);
    }

    pub fn parse_eit(&mut self, eit: &Eit) {
        for eit_item in eit.items.iter() {
            let mut event = EpgEvent::default();
            event.parse_eit(eit_item);
            self.events.push(event);
        }
    }

    pub fn sort(&mut self) {
        self.events.sort_by(|a, b| a.start.cmp(&b.start));
    }
}

#[derive(Default, Debug)]
pub struct Epg {
    pub channels: HashMap<String, EpgChannel>,
}

impl Epg {
    pub fn parse_xml(&mut self, node: &xml::Node) {
        for i in node.iter_child() {
            match i.key.as_str() {
                "channel" => {
                    let id = match i.get_attr("id") {
                        Some(v) => v,
                        None => continue,
                    };

                    self.channels
                        .entry(id.to_string())
                        .or_insert(EpgChannel::default());
                },
                "programme" => {
                    let id = match i.get_attr("channel") {
                        Some(v) => v,
                        None => continue,
                    };

                    let channel = match self.channels.get_mut(id) {
                        Some(v) => v,
                        None => continue,
                    };

                    channel.parse_xml(i);
                },
                _ => (),
            };
        }

        for (_, channel) in self.channels.iter_mut() {
            channel.sort();
        }
    }

    pub fn parse_eit(&mut self, eit: &Eit) {
        let channel = self.channels
            .entry(format!("{:04x}-{:04x}", eit.tsid, eit.pnr))
            .or_insert_with(|| EpgChannel::default());

        channel.parse_eit(eit);
        channel.sort();
    }
}

//

fn parse_date(s: &str) -> u64 {
    let mut x: u64 = 0;

    if s.len() >= 14 {
        // year
        x += match u64::from_str_radix(&s[0 .. 4], 10) {
            Ok(v) => (365 * v) + (v / 4) - (v / 100) + (v / 400),
            _ => 0,
        };
        // month
        x += match u64::from_str_radix(&s[4 .. 6], 10) {
            Ok(v) => (3 * (v + 1) / 5) + (30 * v),
            _ => 0,
        };
        // day
        x += match u64::from_str_radix(&s[6 .. 8], 10) {
            Ok(v) => v,
            _ => 0,
        };
        x -= 719561;
        x *= 86400;

        // hout
        x += match u64::from_str_radix(&s[8 .. 10], 10) {
            Ok(v) => 3600 * v,
            _ => 0,
        };
        // minute
        x += match u64::from_str_radix(&s[10 .. 12], 10) {
            Ok(v) => 60 * v,
            _ => 0,
        };
        // second
        x += match u64::from_str_radix(&s[12 .. 14], 10) {
            Ok(v) => v,
            _ => 0,
        };
    }

    if s.len() >= 20 {
        let mut off: u64 = 0;
        off += match u64::from_str_radix(&s[16 .. 18], 10) {
            Ok(v) => v * 3600,
            _ => 0,
        };
        off += match u64::from_str_radix(&s[18 .. 20], 10) {
            Ok(v) => v * 60,
            _ => 0,
        };
        match &s[15 .. 16] {
            "-" => x += off,
            "+" => x -= off,
            _ => (),
        };
    }

    x
}
