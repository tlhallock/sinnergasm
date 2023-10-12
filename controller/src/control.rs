extern crate rdev;

// pub mod display;
pub mod events;
pub mod handler;
pub mod listener;
pub mod options;
pub mod prison;

use core::panic;

use crate::handler::handle_events;
use crate::listener::listen_to_system;
use anyhow;
use sinnergasm::options::Options;
use tokio::sync::mpsc as tokio_mpsc;
use tokio_stream;
use tokio_stream::wrappers::UnboundedReceiverStream;

use sinnergasm::grpc_client::create_client;
use ui_common::device_display::display_devices;
use ui_common::events::UiEvent;
use ui_common::subscribe::subscribe_to_workspace;
use ui_common::target::send_target_requests;

fn die_early() {
  panic!("Dying early");
}

#[tokio::main]
// #[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> anyhow::Result<()> {
  let options = Options::new("desktop".into());
  let client = create_client(&options).await?;

  let (sender, receiver) = tokio_mpsc::unbounded_channel::<UiEvent>();
  let (control_sender, control_receiver) = tokio_mpsc::unbounded_channel();
  let (target_sender, target_receiver) =
    tokio_mpsc::unbounded_channel::<UiEvent>();

  let receiver_stream = UnboundedReceiverStream::new(control_receiver);
  let mut client_clone = client.clone();
  let network_task = tokio::task::spawn(async move {
    client_clone.control_workspace(receiver_stream).await?;
    println!("Control workspace finished");
    anyhow::Ok(())
  });

  let sender_clone = sender.clone();
  let client_clone = client.clone();
  let options_clone = options.clone();
  let subscribe_task = tokio::task::spawn(async move {
    subscribe_to_workspace(options_clone, client_clone, sender_clone).await
  });

  let options_clone = options.clone();
  let client_clone = client.clone();
  let target_task = tokio::task::spawn(async move {
    println!("Starting target task");
    send_target_requests(receiver, target_sender, client_clone, options_clone)
      .await?;
    anyhow::Ok(())
  });

  let sender_clone = sender.clone();
  let _ = std::thread::spawn(move || {
    listen_to_system(sender_clone)?;
    anyhow::Ok(())
  });

  sender.send(UiEvent::Targetted)?;
  sender.send(UiEvent::Targetted)?;
  sender.send(UiEvent::Targetted)?;

  let forward_task = tokio::task::spawn(async move {
    handle_events(target_receiver, control_sender).await?;
    Ok(())
  });

  display_devices(client, &options, sender).await?;

  println!("Display closed");

  // TODO: cleanly close the connections...
  die_early();

  let futures = vec![forward_task, target_task, subscribe_task, network_task];
  futures::future::join_all(futures).await;

  anyhow::Ok(())
}

// let forward_sender = sender.clone();
// let foward_task = tokio::task::spawn(async move {
//   while let Some(event) = ui_receiver.recv().await {
//     forward_sender.send(event).expect("Unable to send forward ui event");
//   }
// });
