// pub mod protos {
//     include!(concat!(env!("OUT_DIR"), "/sinnergasm.rs"));
// }

pub mod options;

pub mod protos {
  tonic::include_proto!("sinnergasm"); // The string specified here must match the proto package name
}

pub enum UserInputEvent {
  MouseMove(i32, i32),
  Keyboard(i32, i32),
  Wheel(i32, i32),
}
