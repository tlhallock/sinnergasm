extern crate rdev;

// pub mod display;
// pub mod events;
pub mod handler;
pub mod listener;
pub mod options;
// pub mod state;

use crate::handler::configure_control_stream;
use crate::handler::send_control_events;
use crate::listener::listen_to_system;
use sinnergasm::grpc_client::GrpcClient;
use sinnergasm::options::Options;
use sinnergasm::protos as msg;
use tokio::sync::mpsc as tokio_mpsc;
use tokio_stream;
use tokio_stream::wrappers::UnboundedReceiverStream;

use sinnergasm::grpc_client::create_client;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::broadcast::Sender;
use ui_common::device_display::display_devices;
use ui_common::events;
use ui_common::subscribe::launch_subscription_task;
use ui_common::target::launch_send_targets_task;

// finish creating the launch methods
// Rename them to spawn methods

fn die_early() {
  panic!("Dying early");
}

async fn flush_mouse_movements(
  duration: std::time::Duration,
  sender: Sender<events::AppEvent>,
) -> Result<(), anyhow::Error> {
  let mut interval = tokio::time::interval(duration);
  loop {
    interval.tick().await;
    sender.send(events::AppEvent::ControlEvent(events::ControllerEvent::FlushMouse))?;
  }
}

fn launch_control_task(
  client: GrpcClient,
  control_recv: tokio_mpsc::UnboundedReceiver<msg::ControlRequest>,
) -> tokio::task::JoinHandle<anyhow::Result<()>> {
  let receiver_stream = UnboundedReceiverStream::new(control_recv);
  let mut client_clone = client.clone();
  let network_task = tokio::task::spawn(async move {
    client_clone.control_workspace(receiver_stream).await?;
    anyhow::Ok(())
  });
  return network_task;
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let options = Arc::new(Options::new("desktop".into()));
  let client = create_client(&options).await?;

  let (sender, _) = broadcast::channel::<events::AppEvent>(options.capacity);

  let (control_send, control_recv) = tokio_mpsc::unbounded_channel();
  let network_task = launch_control_task(client.clone(), control_recv);
  configure_control_stream(&control_send, &options)?;

  let subscribe_task = launch_subscription_task(options.clone(), client.clone(), sender.clone(), true).await;
  let target_task = launch_send_targets_task(sender.subscribe(), client.clone(), options.clone()).await;

  let sender_clone = sender.clone();
  let _ = std::thread::spawn(move || {
    listen_to_system(sender_clone)?;
    anyhow::Ok(())
  });

  let receiver = sender.subscribe();
  let forward_task = tokio::task::spawn(async move {
    send_control_events(receiver, control_send).await?;
    Ok(())
  });

  let sender_clone = sender.clone();
  let frequency = options.controller_mouse_frequency;
  let flush_task = tokio::task::spawn(async move {
    flush_mouse_movements(frequency, sender_clone).await?;
    anyhow::Ok(())
  });

  // TODO: add this the simulator as well
  let receiver = sender.subscribe();
  let client_clone = client.clone();
  let options_clone = options.clone();
  let upload_task = tokio::spawn(async move {
    if let Err(err) = ui_common::upload::listen_for_uploads(receiver, client_clone, options_clone).await {
      eprintln!("Error listening for uploads: {}", err);
    }
    anyhow::Ok(())
  });

  display_devices(client, &options, sender).await?;

  println!("Display closed");

  // TODO: cleanly close the connections...
  die_early();

  let futures = vec![forward_task, target_task, subscribe_task, network_task, flush_task, upload_task];
  futures::future::join_all(futures).await;

  anyhow::Ok(())
}

// let forward_sender = sender.clone();
// let foward_task = tokio::task::spawn(async move {
//   while let Some(event) = ui_receiver.recv().await {
//     forward_sender.send(event).expect("Unable to send forward ui event");
//   }
// });
