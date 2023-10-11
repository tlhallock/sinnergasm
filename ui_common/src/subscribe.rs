
use sinnergasm::protos as msg;
use sinnergasm::grpc_client::GrpcClient;
use tokio::sync::mpsc as tokio_mpsc;
use sinnergasm::options::Options;

use crate::events::UiEvent;
use anyhow;


pub async fn subscribe_to_workspace(
  options: Options,
  mut client: GrpcClient,
  sender: tokio_mpsc::UnboundedSender<UiEvent>,
) -> Result<(), anyhow::Error> {
  let subscription_request = msg::WorkspaceSubscriptionRequest {
    workspace: options.workspace.clone(),
    device: options.device.clone(),
  };
  let mut subscription = client
    .subscribe_to_workspace(subscription_request)
    .await?.into_inner();
  while let Some(message) = subscription.message().await? {
    if let Some(msg::workspace_event::EventType::TargetUpdate(msg::TargetUpdate { device: targetted_device })) = message.event_type {
      if targetted_device == *options.device {
        continue;
      }
      sender.send(
        UiEvent::Targetted
      ).expect(
        "Unable to send targetted event"
      );
    }
  }
  Ok(())
}
