use std::collections::HashMap;
use chrono::prelude::*;

use xml;
use mpegts::psi::*;

#[derive(Default, Debug, Clone)]
pub struct EpgEvent {
    pub start: i64,
    pub stop: i64,
    pub title: HashMap<String, String>,
    pub subtitle: HashMap<String, String>,
    pub desc: HashMap<String, String>,
}

const XMLTV_FMT: &str = "%Y%m%d%H%M%S %z";

#[inline]
fn parse_date(value: &str) -> i64 {
    match DateTime::parse_from_str(value, XMLTV_FMT) {
        Ok(v) => v.timestamp(),
        Err(_) => 0,
    }
}

impl EpgEvent {
    pub fn parse_xml(node: &xml::Node) -> EpgEvent {
        let mut event = EpgEvent::default();

        for (key, value) in node.iter_attr() {
            match key.as_str() {
                "start" => event.start = parse_date(&value),
                "stop" => event.stop = parse_date(&value),
                _ => (),
            };
        }

        for i in node.iter_child() {
            match i.key.as_str() {
                "title" => {
                    match i.get_attr("lang") {
                        Some(v) => event.title.insert(v.to_string(), i.text.clone()),
                        None => continue,
                    };
                },
                "sub-title" => {
                    match i.get_attr("lang") {
                        Some(v) => event.subtitle.insert(v.to_string(), i.text.clone()),
                        None => continue,
                    };
                },
                "desc" => {
                    match i.get_attr("lang") {
                        Some(v) => event.desc.insert(v.to_string(), i.text.clone()),
                        None => continue,
                    };
                },
                _ => (),
            };
        }

        event
    }

    pub fn parse_eit(eit_item: &EitItem) -> EpgEvent {
        let mut event = EpgEvent::default();

        event.start = eit_item.start;
        event.stop = eit_item.start + eit_item.duration as i64;

        for desc in eit_item.descriptors.iter() {
            match desc {
                Descriptor::Desc4D(v) => {
                    if v.name.len() > 0 {
                        event.title
                            .entry(v.lang.to_string())
                            .or_insert_with(|| String::new())
                            .push_str(v.name.as_str());
                    }

                    if v.text.len() > 0 {
                        event.subtitle
                            .entry(v.lang.to_string())
                            .or_insert_with(|| String::new())
                            .push_str(v.text.as_str());
                    }
                },
                Descriptor::Desc4E(v) => {
                    if v.text.len() > 0 {
                        event.desc
                            .entry(v.lang.to_string())
                            .or_insert_with(|| String::new())
                            .push_str(v.text.as_str());
                    }
                },
                _ => (),
            };
        }

        event
    }

    pub fn assemble_xml(&self) -> xml::Node {
        let mut node = xml::Node::default();
        node.key.push_str("programme");

        node.push_attr("start".to_string(), Utc.timestamp(self.start, 0).format(XMLTV_FMT).to_string());
        node.push_attr("stop".to_string(), Utc.timestamp(self.stop, 0).format(XMLTV_FMT).to_string());

        let push_child = |node: &mut xml::Node, key: &str, items: &HashMap<String, String>| for (lang, value) in items.iter() {
            let mut x = xml::Node::default();
            x.key.push_str(key);
            x.push_attr("lang".to_string(), lang.to_string());
            x.text.push_str(value);
            node.push_child(x);
        };

        push_child(&mut node, "title", &self.title);
        push_child(&mut node, "sub-title", &self.subtitle);
        push_child(&mut node, "desc", &self.desc);

        node
    }

    pub fn assemble_eit(&self, codepage: usize) -> EitItem {
        let mut eit_item = EitItem::default();

        eit_item.start = self.start;
        eit_item.duration = (self.stop - self.start) as i32;
        eit_item.status = 1;

        for (lang, title) in self.title.iter() {
            let subtitle = match self.subtitle.get(lang) {
                Some(v) => v,
                None => "",
            };

            eit_item.descriptors.push(Descriptor::Desc4D(Desc4D {
                lang: lang.to_string(),
                name: title.to_string(),
                text: subtitle.to_string(),
                codepage: codepage,
            }));
        }

        for (lang, desc) in self.desc.iter() {
            eit_item.descriptors.push(Descriptor::Desc4E(Desc4E {
                number: 0,
                last_number: 0,
                lang: lang.to_string(),
                items: Vec::new(),
                text: desc.to_string(),
                codepage: codepage,
            }));
        }

        eit_item
    }
}

#[derive(Default, Debug)]
pub struct EpgChannel {
    pub event_id: usize,
    pub events: Vec<EpgEvent>,
}

impl EpgChannel {
    pub fn parse_eit(&mut self, eit: &Eit) {
        for eit_item in eit.items.iter() {
            self.events.push(EpgEvent::parse_eit(eit_item));
        }

        self.sort();
    }

    pub fn sort(&mut self) {
        self.events.sort_by(|a, b| a.start.cmp(&b.start));
    }

    pub fn assemble_eit(&self, codepage: usize) -> Eit {
        let mut eit = Eit::default();
        eit.table_id = 0x50;

        let current_time = Utc::now().timestamp();

        for event in self.events.iter() {
            let mut eit_item = event.assemble_eit(codepage);
            eit_item.event_id = (self.event_id as usize + eit.items.len()) as u16;
            if current_time >= event.start && current_time < event.stop {
                eit_item.status = 4;
            }
            eit.items.push(eit_item);
        }

        eit
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

                    let channel = self.channels
                        .entry(id.to_string())
                        .or_insert(EpgChannel::default());

                    channel.event_id = match i.get_attr("event_id") {
                        Some(v) => usize::from_str_radix(v, 10).unwrap_or(0),
                        None => 0,
                    };
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

                    channel.events.push(EpgEvent::parse_xml(i));
                },
                _ => (),
            };
        }

        for (_, channel) in self.channels.iter_mut() {
            channel.sort();
        }
    }

    pub fn assemble_xml(&self) -> xml::Node {
        let mut node = xml::Node::default();
        node.key.push_str("tv");
        node.push_attr("generator-info-name".to_string(), "Cesbo EPG".to_string());

        for (id, _channel) in self.channels.iter() {
            let mut x = xml::Node::default();
            x.key.push_str("channel");
            x.push_attr("id".to_string(), id.to_string());
            // TODO: display-name
            node.push_child(x);
        }

        for (id, channel) in self.channels.iter() {
            for event in channel.events.iter() {
                let mut x = event.assemble_xml();
                x.push_attr("id".to_string(), id.to_string());
                node.push_child(x);
            }
        }

        node
    }
}
