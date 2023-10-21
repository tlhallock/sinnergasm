use sinnergasm::grpc_client::GrpcClient;
use sinnergasm::options::Options;
use sinnergasm::protos as msg;

use crate::events;
use cli_clipboard::ClipboardContext;
use cli_clipboard::ClipboardProvider;
use std::sync::Arc;
use tokio::sync::broadcast::Receiver;

pub async fn send_target_requests(
  mut receiver: Receiver<events::AppEvent>,
  mut client: GrpcClient,
  options: Arc<Options>,
) -> Result<(), anyhow::Error> {
  let mut ctx = ClipboardContext::new().expect("Unable to create clipboard context");

  loop {
    match receiver.recv().await? {
      events::AppEvent::SubscriptionEvent(events::SubscriptionEvent::RequestTarget(device)) => {
        let request = msg::TargetRequest {
          workspace: options.workspace.clone(),
          device: device.clone(),
          clipboard: match ctx.get_contents() {
            Ok(contents) => Some(contents),
            Err(err) => {
              eprintln!("Error getting clipboard contents: {}", err);
              None
            }
          },
        };
        if let Err(err) = client.target_device(request).await {
          eprintln!("Error sending target request: {}", err);
        }
      }
      events::AppEvent::Quit => {
        return Ok(());
      }
      events::AppEvent::ControlEvent(_)
      | events::AppEvent::SubscriptionEvent(_)
      | events::AppEvent::SimulationEvent(_) => {}
    }
  }
}
