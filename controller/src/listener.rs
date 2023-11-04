
use rdev;
use ui_common::errors::RDevError;
use ui_common::events;

use tokio::sync::broadcast::Sender;

pub(crate) fn listen_to_system(sender: Sender<events::AppEvent>) -> Result<(), RDevError> {
  rdev::listen(move |event| {
    if matches!(event.event_type, rdev::EventType::KeyPress(rdev::Key::AltGr)) {
      panic!("Need some escape key: AltGr pressed");
    }
    if let Err(e) = sender.send(events::AppEvent::ControlEvent(events::ControllerEvent::RDevEvent(
      event.event_type,
    ))) {
      eprintln!("Error sending rdev event: {:?}", e);
    }
  })?;
  Ok(())
}
