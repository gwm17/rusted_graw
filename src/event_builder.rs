use std::sync::mpsc::{Receiver, Sender};

use crate::error::EventBuilderError;
use crate::graw_frame::GrawFrame;
use crate::event::Event;
use crate::pad_map::PadMap;

#[derive(Debug)]
pub struct EventBuilder {
    frame_queue: Receiver<GrawFrame>,
    event_queue: Sender<Event>,
    current_event_id: u32,
    pad_map: PadMap,
    frame_stack: Vec<GrawFrame>
}

impl EventBuilder {

    pub fn new(frame_queue: Receiver<GrawFrame>, event_queue: Sender<Event>, pad_map: PadMap) -> Self {
        EventBuilder {
            frame_queue,
            event_queue,
            current_event_id: 0,
            pad_map,
            frame_stack: Vec::new()
        }
    }

    pub fn run(&mut self) -> Result<(), EventBuilderError> {
        loop {

            let frame = match self.frame_queue.recv() {
                Ok(f) => f,
                Err(_) => { //Reciever Errors indicate that the sender channel was closed an there is no more data to be read
                    break;
                }
            };

            if frame.header.event_id != self.current_event_id {
                self.send_event()?;
                self.frame_stack.push(frame);
            } else {
                self.frame_stack.push(frame);
            }

        }

        Ok(())
    }

    fn send_event(&mut self) -> Result<(), EventBuilderError> {
        let event: Event = Event::new(&self.pad_map, &self.frame_stack)?;
        match self.event_queue.send(event) {
            Ok(_) => (),
            Err(_) => {
                return Err(EventBuilderError::SendError);
            }
        }
        self.frame_stack.clear();
        Ok(())
    }
}