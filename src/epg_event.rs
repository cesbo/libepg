use std::collections::HashMap;

use chrono::Utc;

use mpegts::{
    psi::{
        EitItem,
        Desc4D,
        Desc4E,
        DescRaw,
    },
    textcode::StringDVB,
};


#[derive(Default, Debug, Clone, PartialEq)]
pub struct EpgEvent {
    /// Unique event identifier
    pub event_id: u16,
    /// Event start time
    pub start: u64,
    /// Event stop tiem (equal to the next event start time)
    pub stop: u64,
    /// Event title list
    pub title: HashMap<String, String>,
    /// Event short description list
    pub subtitle: HashMap<String, String>,
    /// Event description list
    pub desc: HashMap<String, String>,
    /// Codepage
    pub codepage: u8,
    /// Parental Rating
    pub parental_rating: HashMap<[u8; 3], u8>,
}


impl<'a> From<&'a EitItem> for EpgEvent {
    fn from(eit_item: &EitItem) -> Self {
        let mut event = EpgEvent {
            event_id: eit_item.event_id,
            start: eit_item.start,
            stop: eit_item.start + u64::from(eit_item.duration),
            ..Default::default()
        };

        for desc in eit_item.descriptors.iter() {
            match desc.tag() {
                0x4D => {
                    let v = desc.downcast_ref::<Desc4D>();
                    event.title.insert(v.lang.to_string(), v.name.to_string());

                    if ! v.text.is_empty() {
                        event.subtitle
                            .entry(v.lang.to_string())
                            .or_insert_with(String::new)
                            .push_str(&v.text.to_string());
                    }
                },
                0x4E => {
                    let v = desc.downcast_ref::<Desc4E>();
                    if ! v.text.is_empty() {
                        event.desc
                            .entry(v.lang.to_string())
                            .or_insert_with(String::new)
                            .push_str(&v.text.to_string());
                    }
                },
                0x55 => {
                    // TODO: parental_rating_descriptor
                },
                _ => (),
            };
        }

        event
    }
}


impl<'a> From<&'a EpgEvent> for EitItem {
    fn from(event: &EpgEvent) -> Self {
        let mut eit_item = EitItem {
            event_id: event.event_id,
            start: event.start,
            duration: (event.stop - event.start) as u32,
            ..Default::default()
        };

        let current_time = Utc::now().timestamp() as u64;
        if current_time >= event.start && current_time < event.stop {
            eit_item.status = 4;
        } else {
            eit_item.status = 1;
        }

        for (lang, title) in &event.title {
            let subtitle = match event.subtitle.get(lang) {
                Some(v) => v,
                None => "",
            };

            eit_item.descriptors.push(Desc4D {
                lang: StringDVB::from_str(lang, 0),
                name: StringDVB::from_str(title, event.codepage),
                text: StringDVB::from_str(subtitle, event.codepage),
            });
        }

        for (lang, desc) in &event.desc {
            let mut text_list = StringDVB::from_str(desc, event.codepage);
            text_list.truncate(1000);
            let mut text_list = text_list.split(0xF0);
            let mut number: u8 = 0;
            let last_number: u8 = text_list.len() as u8 - 1;

            while ! text_list.is_empty() {
                let text = text_list.remove(0);
                eit_item.descriptors.push(Desc4E {
                    number,
                    last_number,
                    lang: StringDVB::from_str(lang, 0),
                    items: Vec::new(),
                    text,
                });
                number += 1;
            }
        }

        if ! event.parental_rating.is_empty() {
            let mut desc: Vec<u8> = Vec::default();

            for (country, rating) in &event.parental_rating {
                if (4 ..= 18).contains(rating) {
                    desc.extend_from_slice(country);
                    desc.push(*rating - 3);
                }
            }

            if ! desc.is_empty() {
                eit_item.descriptors.push(DescRaw {
                    tag: 0x55,
                    data: desc,
                });
            }
        }

        eit_item
    }
}
