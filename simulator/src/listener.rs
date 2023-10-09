

use std::time::Duration;

use anyhow;
use rdev::simulate;
use sinnergasm::options::Options;
use sinnergasm::protos as msg;
use sinnergasm::protos::virtual_workspaces_client::VirtualWorkspacesClient;
use tokio::time::timeout;
use tokio_stream;
use tonic::metadata::MetadataValue;
use tonic::transport::Channel;
use tonic::{Request, Status};
use sinnergasm::errors::RDevError;

use crate::events::SimulatorEvent;



pub(crate) fn listen_to_mouse(
  sender: tokio::sync::mpsc::UnboundedSender<SimulatorEvent>,
) -> Result<(), RDevError> {
  rdev::listen(move |event| {
    if let rdev::EventType::MouseMove { x, y } = event.event_type {
      if let Err(e) = sender.send(SimulatorEvent::LocalMouseChanged(x, y)) {
        eprintln!("Error: {:?}", e);
      }
    }
  })?;
  Ok(())
}

