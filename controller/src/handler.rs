use anyhow;
use rdev;
use sinnergasm::options::Options;
use sinnergasm::protos as msg;
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc as tokio_mpsc;
use ui_common::events;
use ui_common::translation as tr;

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
        rdev::EventType::KeyPress(key) => msg::user_input_event::Type::KeyPress(tr::rdev_to_msg(&key)),
        rdev::EventType::KeyRelease(key) => msg::user_input_event::Type::KeyRelease(tr::rdev_to_msg(&key)),
        rdev::EventType::ButtonPress(button) => msg::user_input_event::Type::MousePress(tr::mouse_rdev_to_msg(button)),
        rdev::EventType::ButtonRelease(button) => {
          msg::user_input_event::Type::MouseRelease(tr::mouse_rdev_to_msg(button))
        }
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

#[derive(Debug)]
struct ForwardState {
  // Where the mouse started when we began forwarding
  initial_location: (f64, f64),
  // The cumulative delta from the initial location (not accounting for return to initial position)
  virtual_location: (f64, f64),
  // The last virtual location we sent to the server
  sent_location: (f64, f64),
}

impl ForwardState {
  fn new(initial_location: (f64, f64)) -> Self {
    Self {
      initial_location,
      // mouse_location: initial_location,
      virtual_location: (0.0, 0.0),
      sent_location: (0.0, 0.0),
    }
  }

  fn is_simulated_input(&self, next: (f64, f64)) -> bool {
    self.initial_location == next
    // self.initial_location.0 != x || self.initial_location.1 != y
  }

  fn return_to_initial_position(&self) {
    if let Err(err) = rdev::simulate(&rdev::EventType::MouseMove {
      x: self.initial_location.0,
      y: self.initial_location.1,
    }) {
      println!("Failed to simulate mouse move: {:?}", err);
    }
  }

  fn update(&mut self, last: (f64, f64), next: (f64, f64)) {
    if self.is_simulated_input(next) {
      return;
    }
    let delta_x = next.0 - last.0;
    let delta_y = next.1 - last.1;

    self.virtual_location.0 += delta_x;
    self.virtual_location.1 += delta_y;

    self.return_to_initial_position();
  }

  fn maybe_send(&mut self, sender: &tokio_mpsc::UnboundedSender<msg::ControlRequest>) {
    if self.virtual_location == self.sent_location {
      return;
    }

    let delta_x = self.virtual_location.0 - self.sent_location.0;
    let delta_y = self.virtual_location.1 - self.sent_location.1;
    if let Err(err) = sender.send(mouse_move_event(delta_x, delta_y)) {
      eprintln!("Error sending mouse move message: {}", err);
    }

    self.sent_location = self.virtual_location;
  }
}

pub async fn send_control_events(
  mut receiver: Receiver<events::AppEvent>,
  sender: tokio_mpsc::UnboundedSender<msg::ControlRequest>,
) -> Result<(), anyhow::Error> {
  let mut forward_state = Option::<ForwardState>::None;
  let mut last_position = Option::<(f64, f64)>::None;

  loop {
    match receiver.recv().await? {
      events::AppEvent::Quit => {
        println!("Received quit event");
        return Ok(());
      }
      events::AppEvent::ControlEvent(events::ControllerEvent::RDevEvent(rdev::EventType::MouseMove { x, y })) => {
        if let Some(state) = forward_state.as_mut() {
          let next = (x, y);
          let last = last_position.expect("No last position found");
          state.update(last, next);
        }
        last_position = Some((x, y));
      }
      events::AppEvent::ControlEvent(events::ControllerEvent::RDevEvent(rdev_event)) => {
        if let Some(state) = forward_state.as_mut() {
          // Flush mouse location before other events...
          state.maybe_send(&sender);

          if let Err(err) = sender.send(translate_other_events(rdev_event)) {
            eprintln!("Error sending message: {}", err);
          }
        }
      }
      events::AppEvent::ControlEvent(events::ControllerEvent::FlushMouse) => {
        if let Some(state) = forward_state.as_mut() {
          state.maybe_send(&sender);
        }
      }
      events::AppEvent::SubscriptionEvent(events::SubscriptionEvent::Targetted) => {
        println!("Not fowarding events");
        forward_state = None;
      }
      events::AppEvent::SubscriptionEvent(events::SubscriptionEvent::Untargetted) => {
        if let Some(last) = last_position {
          // Move it out of the way of the go to laptop button...
          let last = (last.0, last.1 + 50.0);
          rdev::simulate(&rdev::EventType::MouseMove { x: last.0, y: last.1 })?;

          forward_state = Some(ForwardState::new(last));
          println!("Starting fowarding events");
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
