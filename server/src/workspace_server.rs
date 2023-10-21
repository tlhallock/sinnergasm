use futures::stream::StreamExt;
use tokio::sync::mpsc;

use crate::actors::simulate::SimulationEvent;
use crate::actors::workspace::SubscriptionEvent;
use crate::events;
use sinnergasm::protos as msg;
use sinnergasm::protos::virtual_workspaces_server::VirtualWorkspaces;
use std::pin::Pin;
use tonic::Status;

type SimulationSender = tokio::sync::mpsc::UnboundedSender<SimulationEvent>;
type WorkspaceSender = tokio::sync::mpsc::UnboundedSender<SubscriptionEvent>;

#[derive(Debug)]
pub(crate) struct WorkspaceServer {
  workspace_sender: WorkspaceSender,
  simulation_sender: SimulationSender,
  // workspaces: Actor<events::WorkspaceEvent>,
  the_workspace: msg::Workspace,
}

impl WorkspaceServer {
  pub(crate) fn new(workspace_sender: WorkspaceSender, simulation_sender: SimulationSender) -> Self {
    Self {
      workspace_sender,
      simulation_sender,
      the_workspace: msg::Workspace {
        name: "The Workspace".to_string(),
        controller: "desktop".to_string(),
        target: "".to_string(), // Why can't this be None?
        devices: vec![
          msg::Device {
            name: "desktop".to_string(),
            controller: true,
          },
          msg::Device {
            name: "laptop".to_string(),
            controller: false,
          },
        ],
        monitors: vec![
          msg::Monitor {
            name: "left".to_string(),
            x: 0,
            y: 0,
            w: 1920,
            h: 1080,
            device: "desktop".to_string(),
          },
          msg::Monitor {
            name: "middle".to_string(),
            x: 1920,
            y: 0,
            w: 1920,
            h: 1200,
            device: "desktop".to_string(),
          },
          msg::Monitor {
            name: "right".to_string(),
            x: 3840,
            y: 0,
            w: 1920,
            h: 1080,
            device: "desktop".to_string(),
          },
        ],
      },
    }
  }
}

// fn map_transition(event: events::WorkspaceSubscriptionEvent) -> Result<msg::WorkspaceEvent, Status> {
//   match event {
//     events::WorkspaceSubscriptionEvent::SetTarget(_, _) => Ok(msg::WorkspaceEvent {
//       event_type: Some(msg::workspace_event::EventType::TargetUpdate(msg::TargetUpdate {
//         device: "".into(),
//       })),
//     }),
//   }
// }

#[tonic::async_trait]
impl VirtualWorkspaces for WorkspaceServer {
  type SubscribeToWorkspaceStream =
    Pin<Box<dyn futures_core::Stream<Item = std::result::Result<msg::WorkspaceEvent, tonic::Status>> + Send + 'static>>;
  type SimulateWorkspaceStream = Pin<
    Box<dyn futures_core::Stream<Item = std::result::Result<msg::SimulationEvent, tonic::Status>> + Send + 'static>,
  >;

  async fn create_workspace(
    &self,
    _request: tonic::Request<msg::CreateRequest>,
  ) -> std::result::Result<tonic::Response<msg::CreatedResponse>, tonic::Status> {
    tracing::info!("Create workspace request");
    Err(tonic::Status::internal("Not implemented"))
  }

  async fn list_workspaces(
    &self,
    _request: tonic::Request<msg::ListRequest>,
  ) -> std::result::Result<tonic::Response<msg::WorkspaceList>, tonic::Status> {
    tracing::info!("Listing workspaces");
    Err(tonic::Status::internal("Not implemented"))
  }

  async fn get_workspace(
    &self,
    request: tonic::Request<msg::GetRequest>,
  ) -> std::result::Result<tonic::Response<msg::Workspace>, tonic::Status> {
    let request = request.into_inner();
    tracing::info!("Getting workspace {}", request.name);
    Ok(tonic::Response::new(self.the_workspace.clone()))
  }

  async fn configure_workspace(
    &self,
    _request: tonic::Request<msg::ConfigurationRequest>,
  ) -> std::result::Result<tonic::Response<msg::ConfiguredResponse>, tonic::Status> {
    tracing::info!("Configuring workspace {}", "implement me");
    Err(tonic::Status::internal("Not implemented"))
  }

  async fn delete_workspace(
    &self,
    _request: tonic::Request<msg::DeleteRequest>,
  ) -> std::result::Result<tonic::Response<msg::DeleteResponse>, tonic::Status> {
    tracing::info!("Delete workspace request");
    Err(tonic::Status::internal("Not implemented"))
  }

  async fn target_device(
    &self,
    request: tonic::Request<msg::TargetRequest>,
  ) -> std::result::Result<tonic::Response<msg::TargetResponse>, tonic::Status> {
    let request = request.into_inner();
    let workspace_name = request.workspace;
    let device_name = request.device;
    let clipboard = request.clipboard;
    tracing::info!("Workspace {} will now target {}", workspace_name, device_name);
    if let Err(err) = self.simulation_sender.send(SimulationEvent::TargetEvent(
      workspace_name.clone(),
      device_name.clone(),
    )) {
      println!("Failed to send listener removed event: {:?}", err);
      return Err(tonic::Status::from_error(Box::new(err)));
    }

    if let Err(err) = self.workspace_sender.send(SubscriptionEvent::TargetEvent(
      workspace_name.clone(),
      device_name.clone(),
      clipboard,
    ))
    // SubscriptionEvent::WorkspaceEvent(
    // workspace_name,
    // msg::WorkspaceEvent {
    //   event_type: Some(msg::workspace_event::EventType::TargetUpdate(msg::TargetUpdate {
    //     device: device_name,
    //   })),
    // },
    // ))
    {
      println!("Failed to notify listeners of target update event: {:?}", err);
      return Err(tonic::Status::from_error(Box::new(err)));
    }

    return Ok(tonic::Response::new(msg::TargetResponse {}));
  }

  async fn cancel_simulation(
    &self,
    request: tonic::Request<msg::CancelSimulationRequest>,
  ) -> std::result::Result<tonic::Response<msg::CancelSimulationResponse>, tonic::Status> {
    tracing::info!("Cancel simulation request");
    let request = request.into_inner();
    let workspace_name = request.workspace;
    let device_name = request.device;
    if let Err(err) = self
      .simulation_sender
      .send(SimulationEvent::RemoveSimulator(workspace_name, device_name))
    {
      println!("Failed to send listener removed event: {:?}", err);
      return Err(tonic::Status::from_error(Box::new(err)));
    }
    return Ok(tonic::Response::new(msg::CancelSimulationResponse {}));
  }

  async fn cancel_subscription(
    &self,
    request: tonic::Request<msg::CancelSubscriptionRequest>,
  ) -> std::result::Result<tonic::Response<msg::CancelSubscriptionResponse>, tonic::Status> {
    tracing::info!("Cancel subscription request");
    let request = request.into_inner();
    let workspace_name = request.workspace;
    let device_name = request.device;
    if let Err(err) = self
      .workspace_sender
      .send(SubscriptionEvent::Unsubscribe(workspace_name, device_name))
    {
      println!("Failed to send listener removed event: {:?}", err);
      return Err(tonic::Status::from_error(Box::new(err)));
    }
    return Ok(tonic::Response::new(msg::CancelSubscriptionResponse {}));
  }

  async fn control_workspace(
    &self,
    request: tonic::Request<tonic::Streaming<msg::ControlRequest>>,
  ) -> std::result::Result<tonic::Response<msg::ControlResponse>, tonic::Status> {
    let mut stream = request.into_inner();

    if let Some(Ok(msg::ControlRequest {
      event_type: Some(msg::control_request::EventType::Workspace(msg::ControlWorkspace { workspace, device })),
    })) = stream.next().await
    {
      println!("Device {} will control workspace {}", device, workspace);
      while let Some(req) = stream.next().await {
        if let Ok(msg::ControlRequest {
          event_type: Some(msg::control_request::EventType::InputEvent(input_event)),
        }) = req
        {
          self
            .simulation_sender
            .send(SimulationEvent::SimulationEvent(
              self.the_workspace.name.clone(),
              msg::SimulationEvent {
                input_event: Some(input_event.clone()),
              },
            ))
            .map_err(|e| tonic::Status::aborted(e.to_string()))?;
        } else {
          tracing::info!("Invalid control message");
          return Err(tonic::Status::aborted("Invalid control message"));
        }
      }
      println!("Done with control workspace request");
      Ok(tonic::Response::new(msg::ControlResponse {}))
    } else {
      return Err(tonic::Status::aborted("No messages in control stream"));

      //   "The first control message must be which workspace to control"
      // ));
    }
  }

  async fn simulate_workspace(
    &self,
    request: tonic::Request<msg::SimulateRequest>,
  ) -> std::result::Result<tonic::Response<Self::SimulateWorkspaceStream>, tonic::Status> {
    let request = request.into_inner();
    let workspace_name = request.workspace;
    let device_name = request.device;
    let (sender, receiver) = mpsc::unbounded_channel::<msg::SimulationEvent>();

    println!("Adding device {} as a simulator for {}.", device_name, workspace_name);

    if let Err(err) = self
      .simulation_sender
      .send(SimulationEvent::AddSimulator(workspace_name, device_name, sender))
    {
      return Err(tonic::Status::from_error(Box::new(err)));
    }

    let response_stream = tokio_stream::wrappers::UnboundedReceiverStream::new(receiver).map(Ok::<_, tonic::Status>);
    Ok(tonic::Response::new(Box::pin(response_stream)))
  }

  async fn subscribe_to_workspace(
    &self,
    request: tonic::Request<msg::WorkspaceSubscriptionRequest>,
  ) -> std::result::Result<tonic::Response<Self::SubscribeToWorkspaceStream>, tonic::Status> {
    let request = request.into_inner();
    let workspace_name = request.workspace;
    let device_name = request.device;
    let (sender, receiver) = mpsc::unbounded_channel::<msg::WorkspaceEvent>();

    println!("Adding device {} as a listener for {}.", device_name, workspace_name);

    if let Err(err) = self
      .workspace_sender
      .send(SubscriptionEvent::Subscribe(workspace_name, device_name, sender))
    {
      return Err(tonic::Status::from_error(Box::new(err)));
    }

    let response_stream = tokio_stream::wrappers::UnboundedReceiverStream::new(receiver).map(Ok::<_, tonic::Status>);
    Ok(tonic::Response::new(Box::pin(response_stream)))
  }
}
