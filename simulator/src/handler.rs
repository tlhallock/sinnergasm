use anyhow;
use rdev::simulate;

use sinnergasm::protos as msg;

use tokio::sync::broadcast::Receiver;


use ui_common::events;

fn simulate_input_event(
  desired_position: (f64, f64),
  event: msg::user_input_event::Type,
) -> Result<Option<(f64, f64)>, rdev::SimulateError> {
  match event {
    msg::user_input_event::Type::MouseMove(msg::MouseMoveEvent { delta_x, delta_y }) => {
      let next_position = (desired_position.0 + delta_x, desired_position.1 + delta_y);
      simulate(&rdev::EventType::MouseMove {
        x: next_position.0,
        y: next_position.1,
      })?;
      Ok(Some(next_position))
    }
    msg::user_input_event::Type::MouseButton(msg::MouseButtonEvent {}) => Ok(None),
    msg::user_input_event::Type::Keyboard(msg::KeyboardEvent {}) => Ok(None),
    msg::user_input_event::Type::Wheel(msg::WheelEvent { dx, dy }) => {
      simulate(&rdev::EventType::Wheel {
        delta_x: dx.into(),
        delta_y: dy.into(),
      })?;
      Ok(None)
    }
  }
}

pub(crate) async fn simulate_receiver(mut receiver: Receiver<events::AppEvent>) -> Result<(), anyhow::Error> {
  let mut initial_position = None;
  let mut desired_position = None;

  loop {
    match receiver.recv().await? {
      events::AppEvent::Quit => {
        return Ok(());
      }
      events::AppEvent::ControlEvent(_) => todo!(),
      events::AppEvent::SimulationEvent(events::SimulationEvent::LocalMouseChanged(x, y)) => {
        initial_position = Some((x, y));
      }
      events::AppEvent::SimulationEvent(events::SimulationEvent::SimulateEvent(msg::SimulationEvent {
        input_event: Some(msg::UserInputEvent { r#type: Some(event) }),
      })) => {
        if let Some(current_position) = desired_position {
          // Fail on first error?
          if let Some(next_position) = simulate_input_event(current_position, event)? {
            desired_position = Some(next_position);
          }
        } else {
          println!("No mouse event yet, we do not know the current location of the mouse.");
        }
      }
      events::AppEvent::SubscriptionEvent(events::SubscriptionEvent::Targetted) => {
        desired_position = initial_position;
      }
      events::AppEvent::SubscriptionEvent(events::SubscriptionEvent::Untargetted) => {
        desired_position = None;
      }
      _ => {}
    }
  }
}
