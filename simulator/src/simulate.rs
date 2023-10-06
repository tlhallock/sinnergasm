use anyhow;
use sinnergasm::protos as msg;
use sinnergasm::protos::virtual_workspaces_client::VirtualWorkspacesClient;
use sinnergasm::SECRET_TOKEN;
use tokio_stream::{self, StreamExt};
use tonic::metadata::MetadataValue;
use tonic::transport::Channel;
use tonic::Request;
use rdev::simulate;

// let cert = std::fs::read_to_string("ca.pem")?;
// .tls_config(ClientTlsConfig::new()
//     .ca_certificate(Certificate::from_pem(&cert))
//     .domain_name("example.com".to_string()))?
// .timeout(Duration::from_secs(5))
// .rate_limit(5, Duration::from_secs(1))


async fn simulate_receiver(
  mut receiver: tokio::sync::mpsc::UnboundedReceiver::<msg::SimulationEvent>,
) {
  while let Some(event) = receiver.recv().await {
    println!("Event: {:?}", event);
  }
}



#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let base_url = "http://localhost:50051";
  let channel = Channel::from_static(base_url)
    .concurrency_limit(256)
    .connect()
    .await?;

  let token: MetadataValue<_> = format!("Bearer {SECRET_TOKEN}",).parse()?;
  let mut client = VirtualWorkspacesClient::with_interceptor(
    channel,
    move |mut req: Request<()>| {
      req.metadata_mut().insert("authorization", token.clone());
      Ok(req)
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
