use sinnergasm::protos as msg;
use ui_common::events::UiEvent;
use std::sync::mpsc;

use rdev;


// TODO: there is a grab feature, maybe the prison is not needed!

#[derive(Default)]
pub(crate) struct MouseTracker {
  last_position: Option<(f64, f64)>,
}

impl MouseTracker {
  pub(crate) fn listen(&mut self, event: UiEvent) {
    if let UiEvent::ControlEvent(rdev::EventType::MouseMove { x, y }) = event
    {
      self.last_position = Some((x, y));
    }
  }
}

pub(crate) struct MouseParoleOfficer {
  required_position: (f64, f64),
  // virtual_location: (f64, f64),
}

impl MouseParoleOfficer {
  pub(crate) fn new(last_position: (f64, f64)) -> Self {
    Self {
      required_position: last_position,
      // virtual_location: last_position,
    }
  }

  pub(crate) fn patch(&mut self, (x, y): (f64, f64)) -> msg::MouseMoveEvent {
    let message = msg::MouseMoveEvent {
      delta_x: x - self.required_position.0,
      delta_y: y - self.required_position.1,
    };
    if x == self.required_position.0 && y == self.required_position.1 {
      // Ignore the events we create
      return message;
    }

    if let Err(err) = rdev::simulate(&rdev::EventType::MouseMove {
      x: self.required_position.0,
      y: self.required_position.1,
    }) {
      println!("Failed to simulate mouse move: {:?}", err);
    }

    // let dx = x - self.required_position.0;
    // let dy = y - self.required_position.1;
    // self.virtual_location = (self.virtual_location.0 + dx, self.virtual_location.1 + dy);
    // println!("Virtual location: {:?}", self.virtual_location);
    // msg::MouseMoveEvent {
    //   x: self.virtual_location.0,
    //   y: self.virtual_location.1,
    // }
    message
  }
}

// if let ControlEvent::RDevEvent(rdev::EventType::MouseMove { x, y }) = event {
//   self.last_position = Some((x, y));
//   if self.lock_mouse && self.required_position.is_some() {
//     self.required_position = Some((x, y));
//   }
// }
// if matches!(event, ControlEvent::StartListening) {
//   if self.last_position.is_none() {
//     println!("No mouse position found, ignoring listen event");
//     return;
//   }
//   self.required_position = self.last_position;
//   self.virtual_location = self.last_position;
// }
// fn lock_mouse(
//   receiver: mpsc::Receiver<ControlEvent>,
//   sender: mpsc::Sender<ControlEvent>,
// ) {
//   let mut current_mouse_pos = (0, 0);

//   while let Ok(event) = receiver.recv() {
//     match event {
//       ControlEvent::StartListening => {
//         println!("Start listening");
//       }
//       _ => {}
//     }
//     if let Err(err) = sender.send(event) {
//       println!("Failed to send event to listener: {:?}", err);
//     }
//   }
// }
