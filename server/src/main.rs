pub mod workspaces;

use tonic::transport::Server;

use sinnergasm::protos::virtual_workspaces_server::VirtualWorkspacesServer;

use crate::workspaces::WorkspaceServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let addr = "[::1]:50051".parse()?;
  let workspaces = WorkspaceServer::default();
  Server::builder()
    .add_service(VirtualWorkspacesServer::new(workspaces))
    .serve(addr)
    .await?;
  Ok(())
}
