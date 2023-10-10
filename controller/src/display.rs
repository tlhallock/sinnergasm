use druid::widget::Button;
use druid::widget::Flex;
use druid::widget::Label;

use druid::AppLauncher;
use druid::Data;
use druid::Widget;
use druid::WidgetExt;
use druid::WindowDesc;
use sinnergasm::protos as msg;

use crate::events::ControlEvent;
use std::sync::mpsc::Sender;

#[derive(Clone, Data)]
struct DisplayState {
  listening: bool,
}

fn ui_builder(sender: Sender<ControlEvent>,
  other_devices: Vec<msg::Device>,) -> impl Widget<DisplayState> {
  let label = Label::dynamic(|state: &DisplayState, _| 
    {
      if state.listening {
        "Forwarding".into()
      } else {
        "Not Forwarding".into()
      }
    })
    .padding(5.0)
    .center();

  let mut column = Flex::column();
  for device in other_devices {
    let label = format!("Go to workspace {}", device.name.clone());
    let button_sender = sender.clone();
    let button = Button::new(label).on_click(move |_ctx, _data, _env| {
      button_sender.send(ControlEvent::Target(device.clone())).expect(
        "Unable to go to queue workspace request")
    });
    column.add_child(button);
  }

  // let button = Button::dynamic(|state: &DisplayState, _| {
  //   if state.listening {
  //     "Stop".into()
  //   } else {
  //     "Start".into()
  //   }
  // })
  // .on_click(move |_ctx, state: &mut DisplayState, _env| {
  //   state.listening = !state.listening;
  //   if let Err(err) = sender.send(if state.listening {
  //     ControlEvent::StartListening
  //   } else {
  //     ControlEvent::StopListening
  //   }) {
  //     println!("Error sending start/stop listening: {}", err);
  //   }
  // })
  // .padding(5.0);

  Flex::column().with_child(label).with_child(column)
}

pub fn launch_display(
  sender: Sender<ControlEvent>,
  other_devices: Vec<msg::Device>,
) -> Result<(), druid::PlatformError> {
  let display_state = DisplayState { listening: false };
  let main_window = WindowDesc::new(ui_builder(sender, other_devices));
  AppLauncher::with_window(main_window)
    .log_to_console()
    .launch(display_state)

  // sender.send(ControlEvent::StartListening).unwrap();
}

// The label text will be computed dynamically based on the current locale and count
// let text = LocalizedString::new("hello-counter")
//   .with_arg("Listening", |state: &DisplayState, _env| {
//     if state.listening {
//       "Listening".into()
//     } else {
//       "Not Listening".into()
//     }
//   });
// let button = Button::new("Toggle Listening")
//   .on_click(|_ctx, state: &mut DisplayState, _env| {
//     state.listening = !state.listening;
//     println!("Listening: {}", state.listening);
//   })
//   .padding(5.0);
