

use sinnergasm::{errors::RDevError, options::Options, grpc_client::GrpcClient};

use tokio::sync::mpsc as tokio_mpsc;
use ui_common::events::UiEvent;
use sinnergasm::protos as msg;


pub(crate) fn listen_to_system(
  sender: tokio_mpsc::UnboundedSender<UiEvent>,
) -> Result<(), RDevError> {
  rdev::listen(move |event| {
    if let rdev::EventType::MouseMove { x, y } = event.event_type {
      sender.send(UiEvent::LocalMouseChanged(x, y)).expect("Unable to send mouse event");
    }
  })?;
  Ok(())
}

pub(crate) async fn listen_to_client(
  options: Options,
  mut client: GrpcClient,
  sender: tokio_mpsc::UnboundedSender<UiEvent>,
) -> Result<(), anyhow::Error> {
  let request = msg::SimulateRequest {
    workspace: options.workspace.clone(),
    device: options.device.clone(),
  };
  let mut stream = client.simulate_workspace(request).await?.into_inner();
  while let Some(event) = stream.message().await? {
    sender.send(UiEvent::SimulateEvent(event))?;
  }
  Ok(())
}
