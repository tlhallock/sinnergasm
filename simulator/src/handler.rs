use anyhow;
use rdev::simulate;

use sinnergasm::protos as msg;

use tokio::sync::mpsc as tokio_mpsc;

use ui_common::events::UiEvent;

fn simulate_input_event(
  (mouse_x, mouse_y): (f64, f64),
  event: msg::user_input_event::Type,
) -> Result<(), rdev::SimulateError> {
  match event {
    println!("Old mouse position: {}, {}", mouse_x, mouse_y);
    msg::user_input_event::Type::MouseMove(msg::MouseMoveEvent {
      delta_x,
      delta_y,
    }) => simulate(&rdev::EventType::MouseMove {
      x: mouse_x + delta_x,
      y: mouse_y + delta_y,
    }),
    msg::user_input_event::Type::MouseButton(msg::MouseButtonEvent {}) => {
      Ok(())
    }
    msg::user_input_event::Type::Keyboard(msg::KeyboardEvent {}) => Ok(()),
    msg::user_input_event::Type::Wheel(msg::WheelEvent { dx, dy }) => {
      simulate(&rdev::EventType::Wheel {
        delta_x: dx.into(),
        delta_y: dy.into(),
      })
    }
  }
}

pub(crate) async fn simulate_receiver(
  mut receiver: tokio_mpsc::UnboundedReceiver<UiEvent>,
) -> Result<(), anyhow::Error> {
  let mut current_mouse_position = None;
  while let Some(event) = receiver.recv().await {
    match event {
      UiEvent::LocalMouseChanged(x, y) => {
        current_mouse_position = Some((x, y));
      }
      UiEvent::SimulateEvent(event) => {
        if let Some((mouse_x, mouse_y)) = current_mouse_position {
          if let Some(msg::UserInputEvent { r#type: Some(event)}) = event.input_event {
            // Fail on first error?
            simulate_input_event((mouse_x, mouse_y), event)?
          }
        } else {
          println!("No mouse event yet, we do not know the current location of the mouse.");
        }
      }
      UiEvent::Quit => todo!(),
      UiEvent::RequestTarget(_)
      | UiEvent::Targetted
      | UiEvent::Untargetted
      /*| UiEvent::ClipboardUpdated(_) */ => {}
      UiEvent::ControlEvent(_) => panic!("Message not expected"),
    }
  }
  Ok(())
}
