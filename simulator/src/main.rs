use anyhow;
use sinnergasm::protos as msg;
use sinnergasm::protos::virtual_workspaces_client::VirtualWorkspacesClient;
use sinnergasm::SECRET_TOKEN;
use tokio_stream;
use tonic::metadata::MetadataValue;
use tonic::transport::Channel;
use tonic::Request;

// let cert = std::fs::read_to_string("ca.pem")?;
// .tls_config(ClientTlsConfig::new()
//     .ca_certificate(Certificate::from_pem(&cert))
//     .domain_name("example.com".to_string()))?
// .timeout(Duration::from_secs(5))
// .rate_limit(5, Duration::from_secs(1))

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
    let secret: &str = "secret";
    let request = msg::ListRequest {};
    let response = client.list_workspaces(request).await;
    if let Ok(response) = response {
      println!("Response: {:?}", response);
    }
  }

  let (sender, receiver) =
    tokio::sync::mpsc::unbounded_channel::<msg::ControlRequest>();

  let future = tokio::task::spawn(async move {
    let receiver_stream =
      tokio_stream::wrappers::UnboundedReceiverStream::new(receiver);
    let response = client.control_workspace(receiver_stream).await?;
    println!("Response: {:?}", response.into_inner());
    anyhow::Ok(())
  });

  {
    if let Err(err) = sender.send(msg::ControlRequest {
      input_event: Some(msg::UserInputEvent {
        r#type: Some(msg::user_input_event::Type::Wheel(msg::WheelEvent {
          dx: 10, // or whatever value you need
          dy: 20, // or whatever value you need
        })),
      }),
    }) {
      eprintln!("Error sending message: {}", err);
    } else {
      println!("Message sent");
    }
  }

  drop(sender);

  if let Err(err) = future.await {
    eprintln!("Error: {}", err);
  }

  Ok(())
}
