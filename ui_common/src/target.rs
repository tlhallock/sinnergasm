use sinnergasm::grpc_client::GrpcClient;
use sinnergasm::options::Options;
use sinnergasm::protos as msg;

use crate::download::spawn_download_task;
use crate::events;
use cli_clipboard::ClipboardContext;
use cli_clipboard::ClipboardProvider;
use std::sync::Arc;
use tokio::sync::broadcast::Receiver;

pub async fn launch_send_targets_task(
  receiver: Receiver<events::AppEvent>,
  client: GrpcClient,
  options: Arc<Options>,
) -> tokio::task::JoinHandle<anyhow::Result<()>> {
  let target_task = tokio::task::spawn(async move {
    send_target_requests(receiver, client, options).await?;
    anyhow::Ok(())
  });
  return target_task;
}

async fn send_target_requests(
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
      events::AppEvent::RequestDwonload(device, shared_file) => {
        println!("handler: Sending download request for {:?}", shared_file);
        let _task = spawn_download_task(client.clone(), device, shared_file, options.clone()).await;
        match _task.await {
          Ok(Ok(())) => println!("Task completed successfully"),
          Ok(Err(_)) => println!("Task returned an error"),
          Err(_) => println!("Task panicked"),
        }
      }
      events::AppEvent::ControlEvent(_)
      | events::AppEvent::SubscriptionEvent(_)
      | events::AppEvent::SimulationEvent(_) => {}
    }
  }
}
