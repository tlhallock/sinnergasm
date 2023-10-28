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

use actors::download_manager::DownloadEvent;
use actors::download_manager::DownloadsActor;
use tonic::transport::Server;
use tonic::{metadata::MetadataValue, Request, Status};

use sinnergasm::options::read_token;
use sinnergasm::options::PORT;
use sinnergasm::protos::virtual_workspaces_server::VirtualWorkspacesServer;
use tokio::sync::mpsc as tokio_mpsc;
use tonic_health::ServingStatus;

use crate::workspace_server::WorkspaceServer;

use tonic::transport::Identity;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let subscriber = FmtSubscriber::builder().with_max_level(Level::INFO).finish();
  tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

  let (workspace_send, mut workspace_recv) = tokio_mpsc::unbounded_channel::<SubscriptionEvent>();
  let workspace_task = tokio::task::spawn(async move {
    let mut workspace_actor = WorkspaceActor::default();
    while let Some(event) = workspace_recv.recv().await {
      if matches!(event, SubscriptionEvent::ApplicationClosing) {
        break;
      }
      workspace_actor.receive(event);
    }
  });

  let (sim_send, mut sim_recv) = tokio_mpsc::unbounded_channel::<SimulationEvent>();
  let replication_task = tokio::task::spawn(async move {
    let mut simulation_actor = SimulationActor::default();
    while let Some(event) = sim_recv.recv().await {
      if matches!(event, SimulationEvent::ApplicationClosing) {
        break;
      }
      simulation_actor.receive(event);
    }
  });

  let (download_send, mut download_receive) = tokio_mpsc::unbounded_channel::<DownloadEvent>();
  let download_task = tokio::task::spawn(async move {
    let mut download_manager = DownloadsActor::default();
    while let Some(event) = download_receive.recv().await {
      if matches!(event, DownloadEvent::ApplicationClosing) {
        break;
      }
      download_manager.receive(event);
    }
  });

  let token = read_token();
  let check_auth = move |req: Request<()>| {
    let metadata: MetadataValue<_> = format!("Bearer {}", token).parse().unwrap();
    match req.metadata().get("authorization") {
      Some(t) if metadata == t => Ok(req),
      _ => Err(Status::unauthenticated("No valid auth token")),
    }
  };

  let (mut health_reporter, _health_service) = tonic_health::server::health_reporter();
  health_reporter
    .set_service_status("virtualworkspaces.VirtualWorkspaces", ServingStatus::Serving)
    .await;

  // health_reporter
  //   // .set_serving::<tonic::service::interceptor::InterceptedService<Self, VirtualWorkspacesServer<WorkspaceServer>>>()
  //   .set_serving::<VirtualWorkspacesServer<WorkspaceServer>>()
  //   .await;
  // health_reporter.set_serving::<VirtualWorkspacesServer<WorkspaceServer>>().await;

  let cert = std::fs::read("keys/server.pem").expect("Missing server.pem");
  let key = std::fs::read("keys/server.key").expect("Missing server.key");
  let addr = format!("0.0.0.0:{}", PORT).parse()?;
  let server = WorkspaceServer::new(workspace_send.clone(), sim_send.clone(), download_send.clone());
  let service = VirtualWorkspacesServer::with_interceptor(server, check_auth);
  Server::builder()
    .tls_config(tonic::transport::ServerTlsConfig::new().identity(Identity::from_pem(&cert, &key)))?
    // .add_service(health_service)
    .add_service(service)
    .serve(addr)
    .await?;

  sim_send.send(SimulationEvent::ApplicationClosing)?;
  workspace_send.send(SubscriptionEvent::ApplicationClosing)?;

  drop(sim_send);
  drop(workspace_send);
  drop(download_send);

  workspace_task.await?;
  replication_task.await?;
  download_task.await?;

  Ok(())
}
