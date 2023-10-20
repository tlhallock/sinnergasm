use sinnergasm::{errors::RDevError, grpc_client::GrpcClient, options::Options};

use sinnergasm::protos as msg;
use std::sync::Arc;
use tokio::sync::broadcast::Sender;
use ui_common::events;

pub(crate) fn listen_to_system(sender: Sender<events::AppEvent>) -> Result<(), RDevError> {
  rdev::listen(move |event| {
    if let rdev::EventType::MouseMove { x, y } = event.event_type {
      sender
        .send(events::AppEvent::SimulationEvent(
          events::SimulationEvent::LocalMouseChanged(x, y),
        ))
        .expect("Unable to send mouse event");
    }
  })?;
  Ok(())
}

pub(crate) async fn listen_to_client(
  options: Arc<Options>,
  mut client: GrpcClient,
  sender: Sender<events::AppEvent>,
) -> Result<(), anyhow::Error> {
  let request = msg::SimulateRequest {
    workspace: options.workspace.clone(),
    device: options.device.clone(),
  };
  let mut stream = client.simulate_workspace(request).await?.into_inner();
  while let Some(event) = stream.message().await? {
    sender.send(events::AppEvent::SimulationEvent(
      events::SimulationEvent::SimulateEvent(event),
    ))?;
  }
  Ok(())
}
