
use sinnergasm::protos as msg;


pub(crate) enum SimulatorEvent {
  ReturnControl,
  LocalMouseChanged(f64, f64),
  SimulateEvent(msg::SimulationEvent),
}


pub(crate) enum SimulatorClientEvent {
  TargetDevice(msg::Device),
}