use crate::events::ControlEvent;
use rdev::listen;
use std::sync::mpsc::Sender;

pub fn stream_events(sender: Sender<ControlEvent>) {
  let result = listen(move |event| {
    let result = sender.send(ControlEvent::RDevEvent(event.event_type));
    if let Err(e) = result {
      println!("Error: {:?}", e);
    }
  });
  if let Err(e) = result {
    println!("Error: {:?}", e);
  }
}

// pub fn callback(event: Event) {
//   match &event.event_type {
//   }
//   // Check the clipboard
//   // if let Ok(clipboard_content) = rdev::
//   // Check the clipboard
//   // if let Ok(clipboard_content) = rdev::Clipboard::get() {
//   //     println!("Clipboard content: {}", clipboard_content);
//   // }
// }
