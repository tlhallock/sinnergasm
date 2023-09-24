use futures::stream::StreamExt;
use tokio::sync::mpsc;

use async_stream::stream;
use sinnergasm::protos as msg;
use sinnergasm::protos::virtual_workspaces_server::VirtualWorkspaces;
// use sinnergasm::UserInputEvent;
use std::pin::Pin;

struct ControlWorkspaceStreamImpl {}

struct ReplicateWorkspaceStreamImpl {}

#[derive(Debug)]
pub struct WorkspaceServer {
  input_event_sender: mpsc::Sender<msg::UserInputEvent>,
}

impl Default for WorkspaceServer {
  fn default() -> Self {
    let (input_event_sender, _replicate_rx) = mpsc::channel(100);
    Self {
      input_event_sender: input_event_sender,
    }
  }
}

#[tonic::async_trait]
impl VirtualWorkspaces for WorkspaceServer {
  type TransitionSubscriptionStream = Pin<
    Box<
      dyn futures_core::Stream<Item = std::result::Result<msg::TransitionEvent, tonic::Status>>
        + Send
        + 'static,
    >,
  >;
  type ReplicationSubscriptionStream = Pin<
    Box<
      dyn futures_core::Stream<Item = std::result::Result<msg::ReplicationEvent, tonic::Status>>
        + Send
        + 'static,
    >,
  >;

  // type ControlWorkspaceStream = ControlWorkspaceStreamImpl;
  // type ControlWorkspaceStream = Box<dyn futures_core::Stream<Item = std::result::Result<super::ControllerEvent, tonic::Status>> + Send + 'static>;
  // type ReplicateWorkspaceStream = ReplicateWorkspaceStreamImpl;

  async fn create_workspace(
    &self,
    request: tonic::Request<msg::CreateRequest>,
  ) -> std::result::Result<tonic::Response<msg::CreatedResponse>, tonic::Status> {
    Err(tonic::Status::internal("Not implemented"))
  }

  async fn list_workspaces(
    &self,
    request: tonic::Request<msg::ListRequest>,
  ) -> std::result::Result<tonic::Response<msg::WorkspaceList>, tonic::Status> {
    Err(tonic::Status::internal("Not implemented"))
  }

  async fn get_workspace(
    &self,
    request: tonic::Request<msg::GetRequest>,
  ) -> std::result::Result<tonic::Response<msg::Workspace>, tonic::Status> {
    Err(tonic::Status::internal("Not implemented"))
  }

  async fn configure_workspace(
    &self,
    request: tonic::Request<msg::ConfigurationRequest>,
  ) -> std::result::Result<tonic::Response<msg::ConfiguredResponse>, tonic::Status> {
    Err(tonic::Status::internal("Not implemented"))
  }

  async fn delete_workspace(
    &self,
    request: tonic::Request<msg::DeleteRequest>,
  ) -> std::result::Result<tonic::Response<msg::DeleteResponse>, tonic::Status> {
    Err(tonic::Status::internal("Not implemented"))
  }

  async fn target_device(
    &self,
    request: tonic::Request<msg::TargetRequest>,
  ) -> std::result::Result<tonic::Response<msg::TargetResponse>, tonic::Status> {
    Err(tonic::Status::internal("Not implemented"))
  }

  async fn join_workspace(
    &self,
    request: tonic::Request<msg::JoinRequest>,
  ) -> std::result::Result<tonic::Response<msg::JoinResponse>, tonic::Status> {
    Err(tonic::Status::internal("Not implemented"))
  }

  async fn leave_workspace(
    &self,
    request: tonic::Request<msg::LeaveRequest>,
  ) -> std::result::Result<tonic::Response<msg::LeaveResponse>, tonic::Status> {
    Err(tonic::Status::internal("Not implemented"))
  }

  async fn control_workspace(
    &self,
    request: tonic::Request<tonic::Streaming<msg::ControlRequest>>,
  ) -> std::result::Result<tonic::Response<msg::StreamControlled>, tonic::Status> {
    let mut stream = request.into_inner();
    let sender = self.input_event_sender.clone();
    while let Some(req) = stream.next().await {
      if let Ok(control_request) = req {
        if let Some(input_event) = control_request.input_event {
          let _ = sender.send(input_event).await;
          // .or_else(||tonic::Status::aborted("raboof"))?;
        }
      }
    }

    Ok(tonic::Response::new(msg::StreamControlled {}))
  }

  async fn replication_subscription(
    &self,
    request: tonic::Request<msg::ReplicationRequest>,
  ) -> std::result::Result<tonic::Response<Self::ReplicationSubscriptionStream>, tonic::Status> {
    Err(tonic::Status::internal("Not implemented"))
  }

  async fn transition_subscription(
    &self,
    request: tonic::Request<msg::TransitionRequest>,
  ) -> std::result::Result<tonic::Response<Self::TransitionSubscriptionStream>, tonic::Status> {
    Err(tonic::Status::internal("Not implemented"))
  }
}
