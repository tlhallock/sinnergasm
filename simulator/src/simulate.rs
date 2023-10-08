use std::time::Duration;

use anyhow;
use rdev::simulate;
use sinnergasm::options::Options;
use sinnergasm::protos as msg;
use sinnergasm::protos::virtual_workspaces_client::VirtualWorkspacesClient;
use tokio::time::timeout;
use tokio_stream;
use tonic::metadata::MetadataValue;
use tonic::transport::Channel;
use tonic::{Request, Status};

// let cert = std::fs::read_to_string("ca.pem")?;
// .tls_config(ClientTlsConfig::new()
//     .ca_certificate(Certificate::from_pem(&cert))
//     .domain_name("example.com".to_string()))?
// .timeout(Duration::from_secs(5))
// .rate_limit(5, Duration::from_secs(1))

async fn simulate_receiver(
  mut receiver: tokio::sync::mpsc::UnboundedReceiver<msg::SimulationEvent>,
) {
  while let Some(event) = receiver.recv().await {
    println!("Event: {:?}", event);
  }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let options = Options::new("laptop".into());
  let channel = Channel::from_shared(options.base_url.clone())?
    .concurrency_limit(options.concurrency_limit);
  let connect_future = channel.connect();
  let channel =
    timeout(Duration::from_secs(options.timeout), connect_future).await??;
  let token: MetadataValue<_> = format!("Bearer {}", options.token).parse()?;
  let mut client = VirtualWorkspacesClient::with_interceptor(
    channel,
    move |mut req: Request<()>| {
      req.metadata_mut().insert("authorization", token.clone());
      Ok::<_, Status>(req)
    },
  );

  {
    let request = msg::ListRequest {};
    let response = client.list_workspaces(request).await;
    if let Ok(response) = response {
      println!("Response: {:?}", response);
    }
  }

  let (sender, receiver) =
    tokio::sync::mpsc::unbounded_channel::<msg::SimulationEvent>();

  let relay_task = tokio::task::spawn(async move {
    let request = msg::SimulateRequest {
      workspace: "The Workspace".into(),
      device: "".into(),
    };
    let response = client.simulate_workspace(request).await?;
    let mut stream = response.into_inner();
    while let Ok(event) = stream.message().await {
      match event {
        Some(event) => sender.send(event)?,
        None => break,
      }
    }
    anyhow::Ok(())
  });

  let simulate_task = tokio::task::spawn(async move {
    simulate_receiver(receiver).await;
    anyhow::Ok(())
  });

  simulate_task.await??;

  if let Err(err) = relay_task.await {
    eprintln!("Error: {}", err);
  }

  Ok(())
}
