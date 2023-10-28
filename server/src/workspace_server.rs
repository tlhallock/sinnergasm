use futures::stream::StreamExt;
use tokio::sync::mpsc;

use crate::actors::download_manager::{DownloadEvent, DownloadKey};
use crate::actors::simulate::SimulationEvent;
use crate::actors::workspace::SubscriptionEvent;
use sinnergasm::protos as msg;
use sinnergasm::protos::virtual_workspaces_server::VirtualWorkspaces;
use std::pin::Pin;

type SimulationSender = tokio::sync::mpsc::UnboundedSender<SimulationEvent>;
type WorkspaceSender = tokio::sync::mpsc::UnboundedSender<SubscriptionEvent>;
type DownloadSender = tokio::sync::mpsc::UnboundedSender<DownloadEvent>;

#[derive(Debug)]
pub(crate) struct WorkspaceServer {
  workspace_sender: WorkspaceSender,
  simulation_sender: SimulationSender,
  download_sender: DownloadSender,
  // workspaces: Actor<events::WorkspaceEvent>,
  the_workspace: msg::Workspace,
}

impl WorkspaceServer {
  pub(crate) fn new(
    workspace_sender: WorkspaceSender,
    simulation_sender: SimulationSender,
    download_sender: DownloadSender,
  ) -> Self {
    Self {
      workspace_sender,
      simulation_sender,
      download_sender,
      the_workspace: msg::Workspace {
        name: "The Workspace".to_string(),
        controller: "desktop".to_string(),
        target: "".to_string(), // Why can't this be None?
        devices: vec![
          msg::Device {
            name: "desktop".to_string(),
            controller: true,
            files: vec![msg::SharedFile {
              relative_path: "simulate".into(),
              size: None,
            }],
          },
          msg::Device {
            name: "laptop".to_string(),
            controller: false,
            files: vec![],
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
  type DownloadFileStream = Pin<
    Box<dyn futures_core::Stream<Item = std::result::Result<msg::DownloadResponse, tonic::Status>> + Send + 'static>,
  >;
  type UploadFileStream =
    Pin<Box<dyn futures_core::Stream<Item = std::result::Result<msg::UploadResponse, tonic::Status>> + Send + 'static>>;

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

  async fn download_file(
    &self,
    request: tonic::Request<tonic::Streaming<msg::DownloadRequest>>,
  ) -> std::result::Result<tonic::Response<Self::DownloadFileStream>, tonic::Status> {
    let mut stream = request.into_inner();
    println!("Download file request");
    if let Some(Ok(msg::DownloadRequest {
      r#type: Some(msg::download_request::Type::Initiate(initiate_request)),
    })) = stream.next().await
    {
      let (sender, receiver) = mpsc::unbounded_channel::<msg::DownloadResponse>();
      let download_key = DownloadKey::new2(&initiate_request);

      println!("Initiating download for {:?}", download_key);

      if let Err(err) = self
        .download_sender
        .send(DownloadEvent::CreateConnection(download_key.clone(), sender))
      {
        return Err(tonic::Status::from_error(Box::new(err)));
      }

      println!("Sending download requested to workspace manager");

      if let Err(err) = self.workspace_sender.send(SubscriptionEvent::DownloadRequested(
        initiate_request.workspace.clone(),
        initiate_request,
      )) {
        // The connection is still present...
        return Err(tonic::Status::from_error(Box::new(err)));
      }

      println!("download: handling other messages");

      let download_sender = self.download_sender.clone();
      tokio::task::spawn(async move {
        while let Some(req) = stream.next().await {
          if let Ok(msg::DownloadRequest {
            r#type: Some(download_request),
          }) = req
          {
            match download_request {
              msg::download_request::Type::Initiate(_) => {
                eprintln!("Initiate message should only be sent once");
              }
              msg::download_request::Type::Request(chunk_request) => {
                println!("Sending download chunk request to download manager {}", chunk_request.offset);
                if let Err(err) = download_sender.send(DownloadEvent::RequestFileChunk(
                  download_key.clone(),
                  chunk_request.offset,
                )) {
                  println!("Failed to send download data to download manager: {:?}", err);
                }
              }
              msg::download_request::Type::Complete(_) => {
                println!("Sending download complete to download manager");
                if let Err(err) = download_sender.send(DownloadEvent::DownloadComplete(download_key.clone())) {
                  println!("Failed to send download complete to download manager: {:?}", err);
                }
              }
            }
          } else {
            println!("Invalid download message");
          }
        }
      });

      let response_stream = tokio_stream::wrappers::UnboundedReceiverStream::new(receiver).map(Ok::<_, tonic::Status>);
      Ok(tonic::Response::new(Box::pin(response_stream)))
    } else {
      return Err(tonic::Status::aborted("First message must be initiate"));
    }
  }

  async fn upload_file(
    &self,
    request: tonic::Request<tonic::Streaming<msg::UploadRequest>>,
  ) -> std::result::Result<tonic::Response<Self::UploadFileStream>, tonic::Status> {
    println!("Upload file request");
    let mut stream = request.into_inner();
    if let Some(Ok(msg::UploadRequest {
      r#type: Some(msg::upload_request::Type::Initiate(initiate_request)),
    })) = stream.next().await
    {
      println!("Initiating upload for {:?}", initiate_request);
      let (sender, receiver) = mpsc::unbounded_channel::<msg::UploadResponse>();
      println!("Creating download key");
      let download_key = DownloadKey::new(&initiate_request);
      println!("Download key: {:?}", download_key);
      if let Err(err) = self.download_sender.send(DownloadEvent::ConnectUploader(
        download_key.clone(),
        sender,
        initiate_request,
      )) {
        return Err(tonic::Status::from_error(Box::new(err)));
      }
      println!("Sending upload requested to workspace manager");

      let download_sender = self.download_sender.clone();
      tokio::task::spawn(async move {
        while let Some(req) = stream.next().await {
          println!("Re download request {:?}", req);
          if let Ok(msg::UploadRequest {
            r#type: Some(upload_request),
          }) = req
          {
            match upload_request {
              msg::upload_request::Type::Initiate(_) => {
                eprintln!("Initiate message should only be sent once");
              }
              msg::upload_request::Type::Chunk(chunk_request) => {
                if let Err(err) =
                  download_sender.send(DownloadEvent::SendFileChunk(download_key.clone(), chunk_request))
                {
                  println!("Failed to send upload chunk request to download manager: {:?}", err);
                }
              }
            }
          } else {
            println!("Invalid upload message");
          }
        }
      });

      println!("upload: returning response stream");
      let response_stream = tokio_stream::wrappers::UnboundedReceiverStream::new(receiver).map(Ok::<_, tonic::Status>);
      Ok(tonic::Response::new(Box::pin(response_stream)))
    } else {
      return Err(tonic::Status::aborted("First message must be initiate"));
    }
  }
}
