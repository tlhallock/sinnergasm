use crate::events;
use cli_clipboard::ClipboardContext;
use cli_clipboard::ClipboardProvider;
use sinnergasm::grpc_client::GrpcClient;
use sinnergasm::options::Options;
use sinnergasm::protos as msg;
use std::sync::Arc;
use tokio::sync::broadcast::Sender;

pub enum SubscriptionEvent {
  Targetted,
  Untargetted,
}

pub async fn subscribe_to_workspace(
  options: Arc<Options>,
  mut client: GrpcClient,
  sender: Sender<events::AppEvent>,
  reuest_target: bool,
) -> Result<(), anyhow::Error> {
  let mut ctx = ClipboardContext::new().expect("Unable to create clipboard context");

  let subscription_request = msg::WorkspaceSubscriptionRequest {
    workspace: options.workspace.clone(),
    device: options.device.clone(),
  };

  let mut subscription = client.subscribe_to_workspace(subscription_request).await?.into_inner();

  if reuest_target {
    sender.send(events::AppEvent::target(options.device.clone()))?;
  }

  while let Some(message) = subscription.message().await? {
    println!("Subscription message: {:?}", message);
    if let Some(msg::workspace_event::EventType::Targetted(msg::Targetted { clipboard })) = message.clone().event_type {
      println!("Subscription message targetted: clipboard = {:?}", clipboard);
      if let Some(clipboard) = clipboard {
        ctx.set_contents(clipboard).expect("Unable to set clipboard");
      }
      sender.send(events::AppEvent::targetted())?;
    } else if let Some(msg::workspace_event::EventType::Untargetted(msg::Untargetted { device: _ })) =
      message.clone().event_type
    {
      println!("Subscription message untargetted");
      sender.send(events::AppEvent::untargetted())?;
    }
  }
  Ok(())
}
