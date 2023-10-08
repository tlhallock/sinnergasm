use crate::events::ControlEvent;
use rdev;
use sinnergasm::errors::RDevError;
use std::sync::mpsc::Sender;

pub(crate) fn listen_to_keyboard_and_mouse(
  sender: Sender<ControlEvent>,
) -> Result<(), RDevError> {
  rdev::listen(move |event| {
    let result = sender.send(ControlEvent::RDevEvent(event.event_type));
    if let Err(e) = result {
      eprintln!("Error: {:?}", e);
    }
  })?;
  Ok(())
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
