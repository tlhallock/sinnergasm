use rdev;
use sinnergasm::protos as msg;

#[derive(Debug, Clone)]
pub enum ControllerEvent {
  RDevEvent(rdev::EventType),
  FlushMouse,
}

#[derive(Debug, Clone)]
pub enum SimulationEvent {
  LocalMouseChanged(f64, f64),
  SimulateEvent(msg::SimulationEvent),
}

#[derive(Debug, Clone)]
pub enum SubscriptionEvent {
  Targetted,
  Untargetted,
  RequestTarget(String),
  BeginUpload(msg::UploadRequested),
}

#[derive(Debug, Clone)]
pub enum AppEvent {
  Quit,
  ControlEvent(ControllerEvent),
  SimulationEvent(SimulationEvent),
  SubscriptionEvent(SubscriptionEvent),
}

impl AppEvent {
  pub fn target(device: String) -> Self {
    AppEvent::SubscriptionEvent(SubscriptionEvent::RequestTarget(device))
  }
  pub(crate) fn targetted() -> Self {
    AppEvent::SubscriptionEvent(SubscriptionEvent::Targetted)
  }
  pub(crate) fn untargetted() -> Self {
    AppEvent::SubscriptionEvent(SubscriptionEvent::Untargetted)
  }
}
