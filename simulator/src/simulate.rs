// pub mod display;
pub mod events;
pub mod handler;
pub mod listener;

use ui_common::subscribe::launch_subscription_task;

use anyhow;
use sinnergasm::grpc_client::create_client;
use sinnergasm::options::Options;
use ui_common::device_display::display_devices;
use ui_common::target::launch_send_targets_task;

use crate::handler::simulate_receiver;
use crate::listener::listen_to_client;
use crate::listener::listen_to_system;
use std::sync::Arc;
use tokio::sync::broadcast;

fn die_early() {
  panic!("Dying early");
}

fn print_type_of<T>(_: &T) {
  println!("{}", std::any::type_name::<T>());
}

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> anyhow::Result<()> {
  let options = Arc::new(Options::new("laptop".into()));
  let client = create_client(&options).await?;
  print_type_of(&client);

  let (sender, _) = broadcast::channel(options.capacity);

  let sender_clone = sender.clone();
  let _ = std::thread::spawn(move || listen_to_system(sender_clone));

  let subscribe_task = launch_subscription_task(options.clone(), client.clone(), sender.clone(), false).await;

  let sender_clone = sender.clone();
  let client_clone = client.clone();
  let options_clone = options.clone();
  let relay_task = tokio::task::spawn(async move {
    listen_to_client(options_clone, client_clone, sender_clone).await?;
    Ok(())
  });

  let target_task = launch_send_targets_task(sender.subscribe(), client.clone(), options.clone()).await;

  let receiver = sender.subscribe();
  let simulate_task = tokio::task::spawn(async move {
    simulate_receiver(receiver).await?;
    Ok(())
  });

  display_devices(client, &options, sender).await?;

  // TODO: figure out how to gracefully close the connections...
  die_early();

  let futures = vec![subscribe_task, relay_task, target_task, simulate_task];
  futures::future::join_all(futures).await;

  Ok(())
}
