use std::collections::HashMap;

use mpegts::psi::Eit;

use crate::EpgEvent;


#[derive(Default, Debug)]
pub struct EpgChannel {
    /// Channel names list
    pub name: HashMap<String, String>,
    /// Channel events list
    pub events: Vec<EpgEvent>,
    /// Start time for last event
    pub last_event_start: u64,
}


impl EpgChannel {
    pub fn parse(&mut self, eit: &Eit) {
        for eit_item in &eit.items {
            self.events.push(EpgEvent::from(eit_item));
        }
        self.sort();
    }

    pub fn sort(&mut self) {
        if self.events.is_empty() {
            return;
        }

        self.events.sort_by(|a, b| a.start.cmp(&b.start));

        self.last_event_start = self.events.last().unwrap().start;

        let mut event_id = self.events.first().unwrap().event_id;
        for event in &mut self.events {
            event.event_id = event_id;
            event_id += 1;
        }
    }
}
