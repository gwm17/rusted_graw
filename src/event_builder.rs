
use crate::error::EventBuilderError;
use crate::graw_frame::GrawFrame;
use crate::event::Event;
use crate::pad_map::PadMap;

#[derive(Debug)]
pub struct EventBuilder {
    current_event_id: u32,
    pad_map: PadMap,
    frame_stack: Vec<GrawFrame>
}

impl EventBuilder {

    pub fn new(pad_map: PadMap) -> Self {
        EventBuilder {
            current_event_id: 0,
            pad_map,
            frame_stack: Vec::new()
        }
    }

    pub fn append_frame(&mut self, frame: GrawFrame) -> Result<Option<Event>, EventBuilderError> {

        if frame.header.event_id > self.current_event_id && self.current_event_id != 0 {
            let event: Event = Event::new(&self.pad_map, &self.frame_stack)?;
            self.frame_stack.clear();
            self.current_event_id = frame.header.event_id;
            self.frame_stack.push(frame);

            return Ok(Some(event));
        } else if self.current_event_id == 0 {
            self.current_event_id = frame.header.event_id;
            self.frame_stack.push(frame);
            return Ok(None)
        } else if frame.header.event_id < self.current_event_id {
            return Err(EventBuilderError::EventOutOfOrder(frame.header.event_id, self.current_event_id));
        } else {
            self.frame_stack.push(frame);
            return Ok(None);
        }
    }

    pub fn flush_final_event(&mut self) -> Option<Event> {
        if self.frame_stack.len() != 0 {
            match Event::new(&self.pad_map, &self.frame_stack) {
                Ok(event) => Some(event),
                Err(_) => None
            }
        } else {
            None
        }
    }

}