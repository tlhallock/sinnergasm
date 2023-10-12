use sinnergasm::grpc_client::GrpcClient;
use sinnergasm::options::Options;
use sinnergasm::protos as msg;

use crate::events::UiEvent;
use cli_clipboard::ClipboardContext;
use cli_clipboard::ClipboardProvider;
use tokio::sync::mpsc as tokio_mpsc;

pub async fn send_target_requests(
  mut receiver: tokio_mpsc::UnboundedReceiver<UiEvent>,
  sender: tokio_mpsc::UnboundedSender<UiEvent>,
  mut client: GrpcClient,
  options: Options,
) -> Result<(), anyhow::Error> {
  let mut ctx =
    ClipboardContext::new().expect("Unable to create clipboard context");

  while let Some(event) = receiver.recv().await {
    if let UiEvent::RequestTarget(device) = &event {
      println!("Requesting target: {}", device.name);

      let request = msg::TargetRequest {
        workspace: options.workspace.clone(),
        device: device.name.clone(),
        clipboard: match ctx.get_contents() {
          Ok(contents) => Some(contents),
          Err(err) => {
            eprintln!("Error getting clipboard contents: {}", err);
            None
          }
        },
      };
      client.target_device(request).await?;
    }
    sender.send(event)?;
  }
  println!("Target task finished");
  Ok(())
}
