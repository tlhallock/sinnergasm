
use druid::widget::{Button, Flex, Label};
use druid::AppLauncher;
use druid::Data;
use druid::Widget;
use druid::WidgetExt;
use druid::WindowDesc;

use crate::events::ControlEvent;
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;


pub(crate) enum UIEvent {
  ConnectionLost,
  ConfigurationUpdate,
  Targetted,
}


#[derive(Clone, Data)]
struct DisplayState {
    listening: bool,
    button_count: usize, // holds the number of buttons to be drawn
}

fn ui_builder(sender: Sender<ControlEvent>, receiver: Receiver<UIEvent>) -> impl Widget<DisplayState> {
    // Label for displaying status
    let label = Label::dynamic(|state: &DisplayState, _| {
        if state.listening {
            "Forwarding".into()
        } else {
            "Not Forwarding".into()
        }
    })
    .padding(5.0)
    .center();

    // Start/Stop button
    let button = Button::dynamic(|state: &DisplayState, _| {
        if state.listening {
            "Stop".into()
        } else {
            "Start".into()
        }
    })
    .on_click(move |_ctx, state: &mut DisplayState, _env| {
        state.listening = !state.listening;
        if state.listening {
            sender.send(ControlEvent::StartListening).unwrap();
        } else {
            sender.send(ControlEvent::StopListening).unwrap();
        }
    })
    .padding(5.0);

    // Connect button
    let connect_button = Button::new("Connect")
        .on_click(|_ctx, state: &mut DisplayState, _env| {
            // Handle logic for connection here, e.g., sender.send(ControlEvent::Connect).unwrap();
        })
        .padding(5.0);

    // Dynamic buttons based on `button_count`
    let dynamic_buttons = Flex::column().with_child(connect_button).with_spacer(10.0).with_child(button);

    // Update dynamic_buttons based on button_count
    let mut flex = Flex::column();
    for _ in 0..state.button_count {
        flex.add_child(
          Button::new("Dynamic Button")
            .padding(5.0)); // add your button logic here
    }

    // Add all to main layout
    Flex::column().with_child(label).with_child(dynamic_buttons).with_child(flex)
}

pub fn launch_display(
    sender: Sender<ControlEvent>,
    receiver: Receiver<UIEvent>,  // <-- take in the receiver
) -> Result<(), druid::PlatformError> {
    let display_state = DisplayState {
        listening: false,
        button_count: 0, // initial button count
    };
    let main_window = WindowDesc::new(ui_builder(sender, receiver)); // <-- pass the receiver

    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(display_state)
}

// In your application's logic, you can then listen to the receiver for UIEvents and update the state accordingly:
fn listen_for_ui_events(receiver: Receiver<UIEvent>, state: &mut DisplayState) {
    loop {
        match receiver.recv() {
            Ok(UIEvent::AddButton) => {
                state.button_count += 1;
                // Trigger UI refresh if needed
            }
            Ok(UIEvent::RemoveButton) => {
                if state.button_count > 0 {
                    state.button_count -= 1;
                }
                // Trigger UI refresh if needed
            }
            // Handle other events
            Err(_) => {
                // Handle errors, e.g., break loop if the sender is dropped
                break;
            }
        }
    }
}