use rdev;
use sinnergasm::protos as msg;

#[derive(Debug)]
pub enum ControlEvent {
  Target(msg::Device),
  RDevEvent(rdev::EventType),
  CloseApplication,
  WeBeTargetted, 
}
