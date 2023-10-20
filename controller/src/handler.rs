use crate::state::MouseParoleOfficer;
use anyhow;
use rdev;
use sinnergasm::options::Options;
use sinnergasm::protos as msg;
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc as tokio_mpsc;
use ui_common::events;

pub(crate) fn configure_control_stream(
  control_sender: &tokio_mpsc::UnboundedSender<msg::ControlRequest>,
  options: &Options,
) -> Result<(), anyhow::Error> {
  control_sender.send(msg::ControlRequest {
    event_type: Some(msg::control_request::EventType::Workspace(msg::ControlWorkspace {
      workspace: options.workspace.clone(),
      device: options.device.clone(),
    })),
  })?;
  Ok(())
}

fn translate_other_events(event: rdev::EventType) -> msg::ControlRequest {
  msg::ControlRequest {
    event_type: Some(msg::control_request::EventType::InputEvent(msg::UserInputEvent {
      r#type: Some(match event {
        rdev::EventType::KeyPress(_key) => msg::user_input_event::Type::Keyboard(msg::KeyboardEvent {}),
        rdev::EventType::KeyRelease(_key) => msg::user_input_event::Type::Keyboard(msg::KeyboardEvent {}),
        rdev::EventType::ButtonPress(_button) => msg::user_input_event::Type::MouseButton(msg::MouseButtonEvent {}),
        rdev::EventType::ButtonRelease(_button) => msg::user_input_event::Type::MouseButton(msg::MouseButtonEvent {}),
        rdev::EventType::Wheel { delta_x, delta_y } => msg::user_input_event::Type::Wheel(msg::WheelEvent {
          dx: delta_x as i32,
          dy: delta_y as i32,
        }),
        rdev::EventType::MouseMove { x: _, y: _ } => panic!("Handled seperately"),
      }),
    })),
  }
}

fn mouse_move_event(delta_x: f64, delta_y: f64) -> msg::ControlRequest {
  msg::ControlRequest {
    event_type: Some(msg::control_request::EventType::InputEvent(msg::UserInputEvent {
      r#type: Some(msg::user_input_event::Type::MouseMove(msg::MouseMoveEvent {
        delta_x,
        delta_y,
      })),
    })),
  }
}

// // TODO: Is this still needed?
// tokio::task::yield_now().await;

pub async fn send_control_events(
  mut receiver: Receiver<events::AppEvent>,
  sender: tokio_mpsc::UnboundedSender<msg::ControlRequest>,
) -> Result<(), anyhow::Error> {
  let mut next_position = None;
  let mut last_position = None;
  let mut officer = Option::<MouseParoleOfficer>::None;

  loop {
    match receiver.recv().await? {
      events::AppEvent::Quit => {
        return Ok(());
      }
      events::AppEvent::ControlEvent(events::ControllerEvent::RDevEvent(rdev::EventType::MouseMove { x, y })) => {
        if let Some(officer) = officer.as_mut() {
          next_position = officer.get_delta((x, y));
        } else {
          last_position = Some((x, y));
        }
      }
      events::AppEvent::ControlEvent(events::ControllerEvent::RDevEvent(rdev_event)) => {
        if officer.is_some() {
          if let Err(err) = sender.send(translate_other_events(rdev_event)) {
            eprintln!("Error sending message: {}", err);
          }
        }
      }
      events::AppEvent::ControlEvent(events::ControllerEvent::FlushMouse) => {
        if let Some((delta_x, delta_y)) = next_position.take() {
          if let Err(err) = sender.send(mouse_move_event(delta_x, delta_y)) {
            eprintln!("Error sending mouse move message: {}", err);
          }
        }
      }
      events::AppEvent::SubscriptionEvent(events::SubscriptionEvent::Targetted) => {
        println!("Stopped listening");
        officer = None;
      }
      events::AppEvent::SubscriptionEvent(events::SubscriptionEvent::Untargetted) => {
        if let Some((x, y)) = last_position {
          officer = Some(MouseParoleOfficer::new((x, y)));
          println!("Starting to listen");
        } else {
          println!("No mouse position found, ignoring listen event");
        }
      }
      _ => {}
    }
  }

  // while let Some(event) =  {
  //   match event {
  //     UiEvent::ControlEvent(event_type) => {
  //       if let Some(officer) = officer.as_mut() {
  //         if let Some(translated) = translate_event(officer, event_type) {
  //           if let Err(err) = sender.send(translated) {
  //             eprintln!("Error sending message: {}", err);
  //           }
  //         }
  //         // // TODO: Is this still needed?
  //         // tokio::task::yield_now().await;
  //       } else if let rdev::EventType::MouseMove { x, y } = event_type {
  //         last_position = Some((x, y));
  //       }
  //     }
  //   }
  // }

  // Ok(())
}
