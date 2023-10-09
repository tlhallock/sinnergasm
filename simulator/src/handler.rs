
use std::time::Duration;

use anyhow;
use rdev::simulate;
use sinnergasm::options::Options;
use sinnergasm::protos as msg;
use sinnergasm::protos::virtual_workspaces_client::VirtualWorkspacesClient;
use tokio::time::timeout;
use tokio_stream;
use tonic::metadata::MetadataValue;
use tonic::transport::Channel;
use tonic::{Request, Status};
use sinnergasm::errors::RDevError;

use crate::events::SimulatorEvent;


fn simulate_input_event((mouse_x, mouse_y): (f64, f64), event: msg::user_input_event::Type) -> Result<(), rdev::SimulateError> {
  match event {
    msg::user_input_event::Type::MouseMove(msg::MouseMoveEvent { delta_x, delta_y }) => {
      simulate(&rdev::EventType::MouseMove {
        x: mouse_x + delta_x,
        y: mouse_y + delta_y,
      })
    }
    msg::user_input_event::Type::MouseButton(msg::MouseButtonEvent {}) => {
      Ok(())
    }
    msg::user_input_event::Type::Keyboard(msg::KeyboardEvent {}) => {
      Ok(())
    }
    msg::user_input_event::Type::Wheel(msg::WheelEvent { dx, dy}) => {
      simulate(&rdev::EventType::Wheel {
        delta_x: dx.into(), delta_y: dy.into()
      })
    }
  }
}


pub(crate) async fn simulate_receiver(
  mut receiver: tokio::sync::mpsc::UnboundedReceiver<SimulatorEvent>,
) -> Result<(), rdev::SimulateError> {
  let mut current_mouse_position = None;
  while let Some(event) = receiver.recv().await {
    match event {
      SimulatorEvent::ReturnControl => {

      },
      SimulatorEvent::LocalMouseChanged(x, y) => {
        current_mouse_position = Some((x, y));
      },
      SimulatorEvent::SimulateEvent(event) => {
        if let Some((mouse_x, mouse_y)) = current_mouse_position {
        if let Some(event) = event.input_event {
          if let Some(event) = event.r#type {
            // Fail on first error?
            simulate_input_event((mouse_x, mouse_y), event)?
          }
        }
      } else {
        println!("No mouse event yet, we do not know the current location of the mouse.");
      }
      }
    }
  }
  Ok(())
}

