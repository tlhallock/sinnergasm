
use sinnergasm::options::Options;
use sinnergasm::protos as msg;
use sinnergasm::grpc_client::GrpcClient;

use crate::events::UiEvent;
use tokio::sync::mpsc as tokio_mpsc;

pub async fn send_target_requests(
  mut receiver: tokio_mpsc::UnboundedReceiver<UiEvent>,
  sender: tokio_mpsc::UnboundedSender<UiEvent>,
  mut client: GrpcClient,
  options: Options,
) -> Result<(), anyhow::Error> {
  let mut clipboard: String = "".into();
  while let Some(event) = receiver.recv().await {
    if let UiEvent::ClipboardUpdated(new_clipboard) = &event {
      clipboard = new_clipboard.clone();
    } else if let UiEvent::RequestTarget(device) = &event {
      let request = msg::TargetRequest {
        workspace: options.workspace.clone(),
        device: device.name.clone(),
        clipboard: clipboard.clone(),
      };
      client.target_device(request).await?;
    }
    sender.send(event)?;
  }
  Ok(())
}
