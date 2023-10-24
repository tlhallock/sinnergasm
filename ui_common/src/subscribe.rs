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

  while let Some(msg::WorkspaceEvent {
    event_type: Some(event_type),
  }) = subscription.message().await?
  {
    println!("Subscription message: {:?}", event_type);
    match event_type {
      msg::workspace_event::EventType::Targetted(msg::Targetted { clipboard }) => {
        // This should just be another clipboard listener...
        println!("Targetted, clipboard = {:?}", &clipboard);
        if let Some(clipboard) = clipboard {
          ctx.set_contents(clipboard).expect("Unable to set clipboard");
        }
        sender.send(events::AppEvent::targetted())?;
      }
      msg::workspace_event::EventType::Untargetted(msg::Untargetted { device: _ }) => {
        println!("Untargetted");
        sender.send(events::AppEvent::untargetted())?;
      }
      msg::workspace_event::EventType::DownloadRequest(upload_request) => {
        println!("Received request to upload {:?}", upload_request);
        sender.send(events::AppEvent::SubscriptionEvent(
          events::SubscriptionEvent::BeginUpload(upload_request),
        ))?;
      }
      msg::workspace_event::EventType::DeviceConnected(_)
      | msg::workspace_event::EventType::DeviceDisconnected(_)
      | msg::workspace_event::EventType::TargetUpdate(_)
      | msg::workspace_event::EventType::ConfigurationUpdate(_) => {}
    }
  }
  Ok(())
}
