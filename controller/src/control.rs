
extern crate rdev;

// pub mod display;
pub mod events;
pub mod handler;
pub mod listener;
pub mod options;
pub mod prison;

use crate::handler::handle_events;
use crate::listener::listen_to_system;
use anyhow;
use sinnergasm::options::Options;
use tokio_stream;
use tokio::sync::mpsc as tokio_mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use ui_common::device_display::display_devices;
use ui_common::target::send_target_requests;
use sinnergasm::grpc_client::create_client;
use ui_common::subscribe::subscribe_to_workspace;



#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let options = Options::new("desktop".into());
  let client = create_client(&options).await?;
  
  let (sender, receiver) = tokio_mpsc::unbounded_channel();
  let (control_sender, control_receiver) = tokio_mpsc::unbounded_channel();
  let (target_sender, target_receiver) = tokio_mpsc::unbounded_channel();

  let receiver_stream = UnboundedReceiverStream::new(control_receiver);
  let mut client_clone = client.clone();
  let network_task = tokio::task::spawn(async move {
    client_clone.control_workspace(receiver_stream).await?;
    anyhow::Ok(())
  });

  let sender_clone = sender.clone();
  let client_clone = client.clone();
  let options_clone = options.clone();
  let subscribe_task = tokio::task::spawn(async move {
    subscribe_to_workspace(options_clone, client_clone, sender_clone).await
  });

  let sender_clone = sender.clone();
  let _ = tokio::task::spawn(async move {
    listen_to_system(sender_clone)?;
    anyhow::Ok(())
  });

  let options_clone = options.clone();
  let client_clone = client.clone();
  let target_task = tokio::task::spawn(async move {
    send_target_requests(receiver, target_sender, client_clone, options_clone).await?;
    Ok(())
  });

  let forward_task = tokio::task::spawn(async move {
    handle_events(target_receiver, control_sender).await?;
    Ok(())
  });

  display_devices(client, &options, sender).await?;

  // TODO: cleanly close the connections...
  panic!("Display closed");

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