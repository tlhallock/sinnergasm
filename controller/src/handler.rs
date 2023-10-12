use crate::prison::MouseParoleOfficer;
use anyhow;
use rdev;
use sinnergasm::protos as msg;
use tokio::sync::mpsc as tokio_mpsc;
use ui_common::events::UiEvent;

// fn handle_rdev(event_type: rdev::EventType) {
//   match event_type {
//     rdev::EventType::KeyPress(key) => {
//       println!("Key Pressed: {:?}", key);
//     }
//     rdev::EventType::KeyRelease(key) => {
//       println!("Key Released: {:?}", key);
//     }
//     rdev::EventType::ButtonPress(button) => {
//       println!("Mouse Button Pressed: {:?}", button);
//     }
//     rdev::EventType::ButtonRelease(button) => {
//       println!("Mouse Button Released: {:?}", button);
//     }
//     rdev::EventType::MouseMove { x, y } => {
//       println!("Mouse Moved to: x = {}, y = {}", x, y);
//     }
//     rdev::EventType::Wheel { delta_x, delta_y } => {
//       println!("Mouse wheel {} {}", delta_x, delta_y);
//     }
//   }
// }

fn translate_event(
  officer: &mut MouseParoleOfficer,
  event: rdev::EventType,
) -> msg::ControlRequest {
  msg::ControlRequest {
    input_event: Some(msg::UserInputEvent {
      r#type: Some(match event {
        rdev::EventType::KeyPress(key) => {
          msg::user_input_event::Type::Keyboard(msg::KeyboardEvent {})
        }
        rdev::EventType::KeyRelease(key) => {
          msg::user_input_event::Type::Keyboard(msg::KeyboardEvent {})
        }
        rdev::EventType::ButtonPress(button) => {
          msg::user_input_event::Type::MouseButton(msg::MouseButtonEvent {})
        }
        rdev::EventType::ButtonRelease(button) => {
          msg::user_input_event::Type::MouseButton(msg::MouseButtonEvent {})
        }
        rdev::EventType::MouseMove { x, y } => {
          let msg = officer.patch((x, y));
          msg::user_input_event::Type::MouseMove(msg)
        }
        rdev::EventType::Wheel { delta_x, delta_y } => {
          msg::user_input_event::Type::Wheel(msg::WheelEvent {
            dx: delta_x as i32,
            dy: delta_y as i32,
          })
        }
      }),
    }),
  }
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
          let translated = translate_event(officer, event_type);
          if let Err(err) = sender.send(translated) {
            eprintln!("Error sending message: {}", err);
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
        if let Some((x, y)) = last_position {
          officer = Some(MouseParoleOfficer::new((x, y)));
          println!("Starting to listen");
        } else {
          println!("No mouse position found, ignoring listen event");
        }
      }
      UiEvent::Untargetted => {
        println!("Stopped forwarding");
        officer = None;
      }
      UiEvent::LocalMouseChanged(_, _) => panic!("Message not expected"),
      UiEvent::SimulateEvent(_) => panic!("Message not expected"),
    }
  }

  Ok(())
}
