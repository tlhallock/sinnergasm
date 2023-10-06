use crate::events::ControlEvent;
use rdev;
use sinnergasm::protos as msg;
use std::sync::mpsc;

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

fn translate_event(event: rdev::EventType) -> msg::ControlRequest {
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
          msg::user_input_event::Type::MouseMove(msg::MouseMoveEvent { x, y })
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

pub async fn forward_events(
  receiver: mpsc::Receiver<ControlEvent>,
  sender: tokio::sync::mpsc::UnboundedSender<msg::ControlRequest>,
) -> Result<(), mpsc::RecvError> {
  let mut listening = false;
  loop {
    let event = receiver.recv()?;
    match event {
      ControlEvent::RDevEvent(event_type) => {
        if !listening {
          continue;
        }
        let translated = translate_event(event_type);
        if let Err(err) = sender.send(translated) {
          eprintln!("Error sending message: {}", err);
        }
        tokio::task::yield_now().await;
      }
      ControlEvent::StartListening => {
        println!("Started Listening");
        listening = true;
      }
      ControlEvent::StopListening => {
        println!("Stopped Listening");
        listening = false;
      }
      ControlEvent::CloseApplication => {
        return Ok(());
      }
    }
  }
}
