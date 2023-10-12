use rdev;
use sinnergasm::errors::RDevError;
use ui_common::events::UiEvent;

use tokio::sync::mpsc as tokio_mpsc;

pub(crate) fn listen_to_system(
  sender: tokio_mpsc::UnboundedSender<UiEvent>,
) -> Result<(), RDevError> {
  rdev::listen(move |event| {
    if matches!(
      event.event_type,
      rdev::EventType::KeyPress(rdev::Key::AltGr)
    ) {
      panic!("Need some escape key: AltGr pressed");
    }
    if let Err(e) = sender.send(UiEvent::ControlEvent(event.event_type)) {
      eprintln!("Error sending rdev event: {:?}", e);
    }
  })?;
  Ok(())
}
