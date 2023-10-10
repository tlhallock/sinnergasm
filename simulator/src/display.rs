

use crate::events::SimulatorClientEvent;

use druid::widget::Button;
use druid::widget::Flex;
use druid::widget::Label;

use druid::AppLauncher;
use druid::Data;
use druid::Widget;
use druid::WidgetExt;
use druid::WindowDesc;

use sinnergasm::protos as msg;

use std::sync::mpsc::Sender;
use tokio::sync::mpsc::UnboundedSender;


#[derive(Clone, Data)]
struct DisplayState {
  number_of_workspaces: usize,
}

fn ui_builder(
  sender: UnboundedSender<SimulatorClientEvent>,
  devices: Vec<msg::Device>,
) -> impl Widget<DisplayState> {
  let mut flex = Flex::column();
  for device in devices {
    let label = format!("Go to workspace {}", device.name.clone());
    let button_sender = sender.clone();
    let button = Button::new(label).on_click(move |_ctx, _data, _env| {
      button_sender.send(SimulatorClientEvent::TargetDevice(device.clone())).expect(
        "Unable to go to queue workspace request")
    });
    flex.add_child(button);
  }
  Flex::column().with_child(flex)
}


pub(crate) fn launch_display(
  sender: UnboundedSender<SimulatorClientEvent>,
  devices: Vec<msg::Device>,
) -> Result<(), druid::PlatformError> {
  let display_state = DisplayState { number_of_workspaces: 0 };
  let main_window = WindowDesc::new(ui_builder(sender, devices));
  AppLauncher::with_window(main_window)
      .log_to_console()
      .launch(display_state)
}