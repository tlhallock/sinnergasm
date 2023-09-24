use crate::events::ControlEvent;
use rdev::EventType;
use std::sync::mpsc::Receiver;

pub fn handle_events(receiver: Receiver<ControlEvent>) {
  loop {
    let event = receiver.recv();
    if let Err(e) = event {
      println!("Error: {:?}", e);
      continue;
    }
    if let Ok(event) = event {
      match event {
        ControlEvent::RDevEvent(event_type) => match event_type {
          EventType::KeyPress(key) => {
            println!("Key Pressed: {:?}", key);
          }
          EventType::KeyRelease(key) => {
            println!("Key Released: {:?}", key);
          }
          EventType::ButtonPress(button) => {
            println!("Mouse Button Pressed: {:?}", button);
          }
          EventType::ButtonRelease(button) => {
            println!("Mouse Button Released: {:?}", button);
          }
          EventType::MouseMove { x, y } => {
            println!("Mouse Moved to: x = {}, y = {}", x, y);
          }
          EventType::Wheel { delta_x, delta_y } => {
            println!("Mouse wheel {} {}", delta_x, delta_y);
          }
        },
        ControlEvent::StartListening => {
          println!("Start Listening");
        }
        ControlEvent::StopListening => {
          println!("Stop Listening");
        }
        ControlEvent::CloseApplication => todo!(),
      }
    }
  }
}
