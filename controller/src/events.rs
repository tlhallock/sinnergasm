use rdev;


#[derive(Debug)]
pub enum ControlEvent {
  StartListening,
  StopListening,
  RDevEvent(rdev::EventType),
  CloseApplication,
}
