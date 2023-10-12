use rdev;
use sinnergasm::protos as msg;

#[derive(Debug)]
pub enum UiEvent {
  Quit,
  RequestTarget(msg::Device),

  Targetted,
  Untargetted,

  // ClipboardUpdated(String),

  // Controller events:
  ControlEvent(rdev::EventType),

  // Simulator events:
  LocalMouseChanged(f64, f64),
  SimulateEvent(msg::SimulationEvent),
}
