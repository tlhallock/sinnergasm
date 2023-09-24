extern crate rdev;

pub mod display;
pub mod events;
pub mod handler;
pub mod listener;

// use crate::simple_ui2::run_ui;
// use crate::app_state::ControllerState;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread;

use crate::display::launch_display;
use crate::handler::handle_events;
use crate::listener::stream_events;

use crate::events::ControlEvent;

use rdev::{listen, Event, EventType};
// use crate::app_state::ControllerState;

use druid::widget::{Button, Flex, Label};
use druid::{
  AppLauncher, LocalizedString, PlatformError, Widget, WidgetExt, WindowDesc,
};

fn main() {
  let (mut sender, recceiver) = channel();

  thread::spawn(move || {
    handle_events(recceiver);
  });
  {
    let sender = sender.clone();
    thread::spawn(move || {
      stream_events(sender);
    });
  }
  {
    let sender = sender.clone();
    thread::spawn(move || {
      launch_display(sender);
    });
  }

  // receiver.recv().unwrap();
  // receiver.recv().unwrap();

  // launch_ui(transmitter);
  // launch_listener(receiver);

  // launch_ui(state);
  // launch_listener();

  // _ = run_ui();
}
