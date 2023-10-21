extern crate rdev;

// pub mod display;
// pub mod events;
pub mod handler;
pub mod listener;
pub mod options;
pub mod state;

use crate::handler::configure_control_stream;
use crate::handler::send_control_events;
use crate::listener::listen_to_system;
use sinnergasm::options::Options;
use tokio::sync::mpsc as tokio_mpsc;
use tokio_stream;
use tokio_stream::wrappers::UnboundedReceiverStream;

use sinnergasm::grpc_client::create_client;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::broadcast::Receiver;
use tokio::sync::broadcast::Sender;
use ui_common::device_display::display_devices;
use ui_common::events;
use ui_common::subscribe::subscribe_to_workspace;
use ui_common::target::send_target_requests;

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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let options = Arc::new(Options::new("desktop".into()));
  let client = create_client(&options).await?;

  let (sender, _) = broadcast::channel::<events::AppEvent>(options.capacity);

  let (control_send, control_recv) = tokio_mpsc::unbounded_channel();
  let receiver_stream = UnboundedReceiverStream::new(control_recv);
  let mut client_clone = client.clone();
  let network_task = tokio::task::spawn(async move {
    client_clone.control_workspace(receiver_stream).await?;
    anyhow::Ok(())
  });
  configure_control_stream(&control_send, &options)?;

  let sender_clone = sender.clone();
  let client_clone = client.clone();
  let options_clone = options.clone();
  let subscribe_task = tokio::task::spawn(async move {
    subscribe_to_workspace(options_clone, client_clone, sender_clone, true).await?;
    anyhow::Ok(())
  });

  let options_clone = options.clone();
  let client_clone = client.clone();
  let receiver = sender.subscribe();
  let target_task = tokio::task::spawn(async move {
    send_target_requests(receiver, client_clone, options_clone).await?;
    anyhow::Ok(())
  });

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

  display_devices(client, &options, sender).await?;

  println!("Display closed");

  // TODO: cleanly close the connections...
  die_early();

  let futures = vec![forward_task, target_task, subscribe_task, network_task, flush_task];
  futures::future::join_all(futures).await;

  anyhow::Ok(())
}

// let forward_sender = sender.clone();
// let foward_task = tokio::task::spawn(async move {
//   while let Some(event) = ui_receiver.recv().await {
//     forward_sender.send(event).expect("Unable to send forward ui event");
//   }
// });
