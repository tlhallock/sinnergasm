use rdev::EventType;

pub enum ControlEvent {
  StartListening,
  StopListening,
  RDevEvent(EventType),
  CloseApplication,
}
