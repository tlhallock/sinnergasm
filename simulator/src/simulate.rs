// pub mod display;
pub mod events;
pub mod handler;
pub mod listener;

use ui_common::subscribe::subscribe_to_workspace;

use anyhow;
use sinnergasm::grpc_client::create_client;
use sinnergasm::options::Options;
use ui_common::device_display::display_devices;
use ui_common::target::send_target_requests;

use crate::handler::simulate_receiver;
use crate::listener::listen_to_client;
use crate::listener::listen_to_system;
use std::sync::Arc;
use tokio::sync::broadcast;

// let cert = std::fs::read_to_string("ca.pem")?;
// .tls_config(ClientTlsConfig::new()
//     .ca_certificate(Certificate::from_pem(&cert))
//     .domain_name("example.com".to_string()))?
// .timeout(Duration::from_secs(5))
// .rate_limit(5, Duration::from_secs(1))

fn die_early() {
  panic!("Dying early");
}

fn print_type_of<T>(_: &T) {
  println!("{}", std::any::type_name::<T>());
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let options = Arc::new(Options::new("laptop".into()));
  let client = create_client(&options).await?;
  print_type_of(&client);

  let (sender, _) = broadcast::channel(options.capacity);

  let sender_clone = sender.clone();
  let _ = std::thread::spawn(move || listen_to_system(sender_clone));

  let sender_clone = sender.clone();
  let client_clone = client.clone();
  let options_clone = options.clone();
  let subscribe_task =
    tokio::task::spawn(async move { subscribe_to_workspace(options_clone, client_clone, sender_clone, false).await });

  let sender_clone = sender.clone();
  let client_clone = client.clone();
  let options_clone = options.clone();
  let relay_task = tokio::task::spawn(async move {
    listen_to_client(options_clone, client_clone, sender_clone).await?;
    Ok(())
  });

  let client_clone = client.clone();
  let options_clone = options.clone();
  let receiver = sender.subscribe();
  let target_task = tokio::task::spawn(async move {
    send_target_requests(receiver, client_clone, options_clone).await?;
    anyhow::Ok(())
  });

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
