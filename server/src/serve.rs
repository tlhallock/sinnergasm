// pub mod workspace;
// pub mod actor;
pub mod actors;
pub mod common;
pub mod events;
pub mod workspace_server;

use crate::actors::simulate::SimulationActor;
use crate::actors::simulate::SimulationEvent;
use crate::actors::workspace::SubscriptionEvent;
use crate::actors::workspace::WorkspaceActor;

use tonic::transport::Server;
use tonic::{metadata::MetadataValue, Request, Status};

use sinnergasm::protos::virtual_workspaces_server::VirtualWorkspacesServer;
use sinnergasm::SECRET_TOKEN;

use crate::workspace_server::WorkspaceServer;

// let cert = std::fs::read_to_string("server.pem")?;
// let key = std::fs::read_to_string("server.key")?;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let (workspace_send, mut workspace_recv) =
    tokio::sync::mpsc::unbounded_channel::<SubscriptionEvent>();
  let workspace_task = tokio::task::spawn(async move {
    let mut workspace_actor = WorkspaceActor::default();
    while let Some(event) = workspace_recv.recv().await {
      if matches!(event, SubscriptionEvent::ApplicationClosing) {
        break;
      }
      workspace_actor.receive(event);
    }
  });

  let (sim_send, mut sim_recv) =
    tokio::sync::mpsc::unbounded_channel::<SimulationEvent>();
  let replication_task = tokio::task::spawn(async move {
    let mut simulation_actor = SimulationActor::default();
    while let Some(event) = sim_recv.recv().await {
      println!("Sending event to actor 125246");
      if matches!(event, SimulationEvent::ApplicationClosing) {
        break;
      }
      simulation_actor.receive(event);
    }
  });

  let addr = "[::1]:50051".parse()?;
  let server = WorkspaceServer::new(workspace_send.clone(), sim_send.clone());

  let service = VirtualWorkspacesServer::with_interceptor(server, check_auth);

  Server::builder()
    // .tls_config(ServerTlsConfig::new()
    //   .identity(Identity::from_pem(&cert, &key)))?
    .add_service(service)
    .serve(addr)
    .await?;

  sim_send.send(SimulationEvent::ApplicationClosing)?;
  workspace_send.send(SubscriptionEvent::ApplicationClosing)?;

  drop(sim_send);
  drop(workspace_send);

  workspace_task.await?;
  replication_task.await?;

  Ok(())
}

fn check_auth(req: Request<()>) -> Result<Request<()>, Status> {
  let token: MetadataValue<_> =
    format!("Bearer {}", SECRET_TOKEN).parse().unwrap();

  match req.metadata().get("authorization") {
    Some(t) if token == t => Ok(req),
    _ => Err(Status::unauthenticated("No valid auth token")),
  }
}
