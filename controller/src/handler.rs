use crate::prison::MouseParoleOfficer;
use anyhow;
use rdev;
use sinnergasm::options::Options;
use sinnergasm::protos as msg;
use tokio::sync::mpsc as tokio_mpsc;
use ui_common::events::UiEvent;

pub(crate) fn configure_control_stream(
  control_sender: &tokio_mpsc::UnboundedSender<msg::ControlRequest>,
  options: &Options,
) -> Result<(), anyhow::Error> {
  control_sender.send(msg::ControlRequest {
    event_type: Some(msg::control_request::EventType::Workspace(
      msg::ControlWorkspace {
        workspace: options.workspace.clone(),
        device: options.device.clone(),
      },
    )),
  })?;
  Ok(())
}

fn translate_input_event(
  officer: &mut MouseParoleOfficer,
  event: rdev::EventType,
) -> Option<msg::user_input_event::Type> {
  match event {
    rdev::EventType::KeyPress(_key) => {
      Some(msg::user_input_event::Type::Keyboard(msg::KeyboardEvent {}))
    }
    rdev::EventType::KeyRelease(_key) => {
      Some(msg::user_input_event::Type::Keyboard(msg::KeyboardEvent {}))
    }
    rdev::EventType::ButtonPress(_button) => Some(
      msg::user_input_event::Type::MouseButton(msg::MouseButtonEvent {}),
    ),
    rdev::EventType::ButtonRelease(_button) => Some(
      msg::user_input_event::Type::MouseButton(msg::MouseButtonEvent {}),
    ),
    rdev::EventType::MouseMove { x, y } => officer
      .patch((x, y))
      .map(|msg| msg::user_input_event::Type::MouseMove(msg)),
    rdev::EventType::Wheel { delta_x, delta_y } => {
      Some(msg::user_input_event::Type::Wheel(msg::WheelEvent {
        dx: delta_x as i32,
        dy: delta_y as i32,
      }))
    }
  }
}

fn translate_event(
  officer: &mut MouseParoleOfficer,
  event: rdev::EventType,
) -> Option<msg::ControlRequest> {
  translate_input_event(officer, event).map(|event_type| msg::ControlRequest {
    event_type: Some(msg::control_request::EventType::InputEvent(
      msg::UserInputEvent {
        r#type: Some(event_type),
      },
    )),
  })
}

pub async fn handle_events(
  mut receiver: tokio_mpsc::UnboundedReceiver<UiEvent>,
  sender: tokio_mpsc::UnboundedSender<msg::ControlRequest>,
) -> Result<(), anyhow::Error> {
  let mut last_position = None;
  let mut officer = None;

  while let Some(event) = receiver.recv().await {
    match event {
      UiEvent::ControlEvent(event_type) => {
        if let Some(officer) = officer.as_mut() {
          if let Some(translated) = translate_event(officer, event_type) {
            if let Err(err) = sender.send(translated) {
              eprintln!("Error sending message: {}", err);
            }
          }
          // // TODO: Is this still needed?
          // tokio::task::yield_now().await;
        } else if let rdev::EventType::MouseMove { x, y } = event_type {
          last_position = Some((x, y));
        }
      }
      // UiEvent::ClipboardUpdated(_) => {}
      UiEvent::RequestTarget(_) => {}
      UiEvent::Quit => {
        return Ok(());
      }
      UiEvent::Targetted => {
        println!("Stopped forwarding");
        officer = None;
      }
      UiEvent::Untargetted => {
        if let Some((x, y)) = last_position {
          officer = Some(MouseParoleOfficer::new((x, y)));
          println!("Starting to listen");
        } else {
          println!("No mouse position found, ignoring listen event");
        }
      }
      UiEvent::LocalMouseChanged(_, _) => panic!("Message not expected"),
      UiEvent::SimulateEvent(_) => panic!("Message not expected"),
    }
  }

  Ok(())
}
