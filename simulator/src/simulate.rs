
pub mod display;
pub mod events;
pub mod handler;
pub mod listener;

use std::time::Duration;
use display::launch_display;

use anyhow;
use sinnergasm::options::Options;
use sinnergasm::protos as msg;
use sinnergasm::protos::virtual_workspaces_client::VirtualWorkspacesClient;
use tokio::time::timeout;
use tonic::metadata::MetadataValue;
use tonic::transport::Channel;
use tonic::{Request, Status};
use crate::events::SimulatorEvent;
use crate::events::SimulatorClientEvent;

use crate::handler::simulate_receiver;
use crate::listener::listen_to_mouse;

// let cert = std::fs::read_to_string("ca.pem")?;
// .tls_config(ClientTlsConfig::new()
//     .ca_certificate(Certificate::from_pem(&cert))
//     .domain_name("example.com".to_string()))?
// .timeout(Duration::from_secs(5))
// .rate_limit(5, Duration::from_secs(1))

fn print_type_of<T>(_: &T) {
  println!("{}", std::any::type_name::<T>());
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let options = Options::new("laptop".into());
  let channel = Channel::from_shared(options.base_url.clone())?
    .concurrency_limit(options.concurrency_limit);
  let connect_future = channel.connect();
  let channel = timeout(Duration::from_secs(options.timeout), connect_future).await??;
  let token: MetadataValue<_> = format!("Bearer {}", options.token).parse()?;
  let mut client = VirtualWorkspacesClient::with_interceptor(
    channel,
    move |mut req: Request<()>| {
      req.metadata_mut().insert("authorization", token.clone());
      Ok::<_, Status>(req)
    },
  );
  print_type_of(&client);

  let devices = {
    let request = msg::GetRequest { name: options.workspace.clone(), };
    let workspace = client.get_workspace(request).await?.into_inner();
    println!("Connecting to workspace: {:?}", workspace);
    workspace.devices
  };


  // Don't need this one, the client is clone
  // TODO: remove this channel
  let (grpc_sender, mut grpc_receiver) = tokio::sync::mpsc::unbounded_channel::<SimulatorClientEvent>();

  let mut sender_client = client.clone();
  let workspace_name = options.workspace.clone();
  let grpc_task = tokio::task::spawn(async move {
    while let Some(message) = grpc_receiver.recv().await {
      match message {
        SimulatorClientEvent::TargetDevice(device) => {
          let request = msg::TargetRequest {
            workspace: workspace_name.clone(),
            device: device.name,
            clipboard: "".into(),
          };
          sender_client.target_device(request).await?;
        }
      }
    }
    anyhow::Ok(())
  });


  let display_sender = grpc_sender.clone();
  let display_task = tokio::task::spawn(async move {
    launch_display(display_sender, devices)?;
    anyhow::Ok(())
  });

  let (sender, receiver) = tokio::sync::mpsc::unbounded_channel::<SimulatorEvent>();

  let key_sender = sender.clone();
  let _ = tokio::task::spawn(async move { listen_to_mouse(key_sender) });

  let relay_task = tokio::task::spawn(async move {
    let request = msg::SimulateRequest {
      workspace: "The Workspace".into(),
      device: "".into(),
    };
    let response = client.simulate_workspace(request).await?;
    let mut stream = response.into_inner();
    while let Ok(event) = stream.message().await {
      match event {
        Some(event) => sender.send(SimulatorEvent::SimulateEvent(event))?,
        None => break,
      }
    }
    anyhow::Ok(())
  });

  let simulate_task = tokio::task::spawn(async move {
    simulate_receiver(receiver).await?;
    anyhow::Ok(())
  });

  display_task.await??;
  println!("Display task finished");

  simulate_task.await??;
  relay_task.await??;
  grpc_task.await??;

  Ok(())
}
