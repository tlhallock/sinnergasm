use rdev;
use sinnergasm::errors::RDevError;
use ui_common::events::UiEvent;

use tokio::sync::mpsc as tokio_mpsc;


pub(crate) fn listen_to_system(
  sender: tokio_mpsc::UnboundedSender<UiEvent>,
) -> Result<(), RDevError> {
  rdev::listen(move |event| {
    // tokio::task::yield_now().await;
    let result = sender.send(UiEvent::ControlEvent(event.event_type));
    if let Err(e) = result {
      eprintln!("Error sending rdev event: {:?}", e);
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
