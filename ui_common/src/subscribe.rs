use cli_clipboard::ClipboardContext;
use cli_clipboard::ClipboardProvider;
use sinnergasm::grpc_client::GrpcClient;
use sinnergasm::options::Options;
use sinnergasm::protos as msg;
use tokio::sync::mpsc as tokio_mpsc;

use crate::events::UiEvent;


pub async fn subscribe_to_workspace(
  options: Options,
  mut client: GrpcClient,
  sender: tokio_mpsc::UnboundedSender<UiEvent>,
) -> Result<(), anyhow::Error> {
  let mut ctx =
    ClipboardContext::new().expect("Unable to create clipboard context");
  let subscription_request = msg::WorkspaceSubscriptionRequest {
    workspace: options.workspace.clone(),
    device: options.device.clone(),
  };
  let mut subscription = client
    .subscribe_to_workspace(subscription_request)
    .await?
    .into_inner();
  while let Some(message) = subscription.message().await? {
    let mut targetted = false;
    if let Some(msg::workspace_event::EventType::Targetted(msg::Targetted {
      clipboard,
    })) = message.clone().event_type
    {
      println!("Subscription message device targetted: {:?}", clipboard);
      if let Some(clipboard) = clipboard {
        ctx
          .set_contents(clipboard)
          .expect("Unable to set clipboard");
      }
      targetted = true;
    };

    // TODO: remove this when targetted is implemented
    if let Some(msg::workspace_event::EventType::TargetUpdate(
      msg::TargetUpdate {
        device: targetted_device,
      },
    )) = message.event_type
    {
      println!(
        "Subscription message device targetted: {}",
        targetted_device
      );
      targetted = targetted_device == *options.device;
    }

    if targetted {
      sender
        .send(UiEvent::Targetted)
        .expect("Unable to send targetted event");
    }
  }
  Ok(())
}
