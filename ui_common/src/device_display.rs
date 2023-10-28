use druid::widget::Button;
use druid::widget::Flex;
use druid::widget::Label;

use crate::events;
use druid::AppLauncher;
use druid::Data;
use druid::Widget;
use druid::WidgetExt;
use druid::WindowDesc;
use sinnergasm::grpc_client::GrpcClient;
use sinnergasm::options::Options;
use sinnergasm::protos as msg;
use tokio::sync::broadcast::Sender;

#[derive(Clone, Data)]
struct DisplayState {
  listening: bool,
}

fn ui_builder(other_devices: Vec<msg::Device>, sender: Sender<events::AppEvent>) -> impl Widget<DisplayState> {
  let label = Label::dynamic(|state: &DisplayState, _| {
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
    let label = format!("Go to device {}", device.name.clone());
    let button_sender = sender.clone();

    let device_name = device.name.clone();
    let button = Button::new(label).on_click(move |_ctx, _data, _env| {
      button_sender
        .send(events::AppEvent::target(device_name.clone()))
        .expect("Unable to go to queue workspace request");
    });
    column.add_child(button);

    for shared_file in device.files {
      let button_sender = sender.clone();
      let device_name = device.name.clone();
      let label = format!(
        "Download shared file {}:{}",
        device_name,
        shared_file.relative_path.clone()
      );
      let button = Button::new(label).on_click(move |_ctx, _data, _env| {
        println!("Sending app event to download {:?}", shared_file);
        button_sender
          .send(events::AppEvent::RequestDwonload(
            device_name.clone(),
            shared_file.clone(),
          ))
          .expect("Unable to go to queue download request");
      });
      column.add_child(button);
    }
  }
  Flex::column().with_child(label).with_child(column)
}

pub async fn display_devices(
  mut client: GrpcClient,
  options: &Options,
  sender: Sender<events::AppEvent>,
) -> Result<(), anyhow::Error> {
  let other_devices = {
    let request = msg::GetRequest {
      name: options.workspace.clone(),
    };
    let workspace = client.get_workspace(request).await?.into_inner();
    println!("Connecting to workspace: {:?}", workspace);
    workspace
      .devices
      .iter()
      .filter(|device| device.name != *options.device)
      .cloned()
      .collect::<Vec<_>>()
  };
  let display_state = DisplayState { listening: false };
  let ui = ui_builder(other_devices, sender);
  let main_window = WindowDesc::new(ui);
  AppLauncher::with_window(main_window)
    .log_to_console()
    .launch(display_state)?;
  Ok(())
}
