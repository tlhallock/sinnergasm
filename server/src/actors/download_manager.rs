use std::collections::BTreeMap;

use tokio::sync::mpsc::UnboundedSender as Sender;

use crate::common as ids;
use sinnergasm::protos as msg;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct DownloadKey {
  pub(crate) workspace: ids::WorkspaceName,
  pub(crate) download_device: ids::DeviceName,
  pub(crate) upload_device: ids::DeviceName,
  pub(crate) relative_path: String,
}

impl DownloadKey {
  pub(crate) fn new(initiate: &msg::InitiateUpload) -> Self {
    Self {
      workspace: initiate.workspace.clone(),
      download_device: initiate.download_device.clone(),
      upload_device: initiate.upload_device.clone(),
      relative_path: initiate.relative_path.clone(),
    }
  }
  pub(crate) fn new2(initiate: &msg::InitiateDownload) -> Self {
    Self {
      workspace: initiate.workspace.clone(),
      download_device: initiate.download_device.clone(),
      upload_device: initiate.upload_device.clone(),
      relative_path: initiate.relative_path.clone(),
    }
  }
}

pub(crate) enum DownloadEvent {
  CreateConnection(DownloadKey, Sender<msg::DownloadResponse>),
  ConnectUploader(DownloadKey, Sender<msg::UploadResponse>, msg::InitiateUpload),
  RequestFileChunk(DownloadKey, u64),
  SendFileChunk(DownloadKey, msg::SharedFileChunk),
  DownloadComplete(DownloadKey),
  ApplicationClosing,
  // DownloadInitiated(ids::WorkspaceName, ids::DeviceName, ids::SharedFileId),
  // Subscribe(
  //   ids::WorkspaceName,
  //   ids::DeviceName,
  //   tokio::sync::mpsc::UnboundedSender<msg::WorkspaceEvent>,
  // ),
  // Unsubscribe(ids::WorkspaceName, ids::DeviceName),
  // WorkspaceEvent(ids::WorkspaceName, msg::WorkspaceEvent),
  // WorskpaceClosing(ids::WorkspaceName),
  // ApplicationClosing,
  // TargetEvent(ids::WorkspaceName, ids::DeviceName, Option<String>),
}

#[derive(Debug)]
pub(crate) struct DownloadConnection {
  download_sender: Sender<msg::DownloadResponse>,
  upload_sender: Option<Sender<msg::UploadResponse>>,
}

impl DownloadConnection {
  fn new(download_sender: Sender<msg::DownloadResponse>) -> Self {
    Self {
      download_sender,
      upload_sender: None,
    }
  }

  fn set_upload_sender(&mut self, upload_sender: Sender<msg::UploadResponse>) {
    self.upload_sender = Some(upload_sender);
  }
}

#[derive(Debug, Default)]
pub(crate) struct DownloadsActor {
  connections: BTreeMap<DownloadKey, DownloadConnection>,
}

impl DownloadsActor {
  pub(crate) fn receive(&mut self, download_event: DownloadEvent) {
    match download_event {
      DownloadEvent::CreateConnection(key, download_sender) => {
        println!("Created connection for key {:?}", key);
        let _ = self
          .connections
          .entry(key)
          .or_insert_with(|| DownloadConnection::new(download_sender));
      }
      DownloadEvent::ConnectUploader(key, upload_sender, request) => {
        if let Some(connection) = self.connections.get_mut(&key) {
          connection.set_upload_sender(upload_sender);

          println!("Sending download initiated for key {:?}", key);
          if let Err(err) = connection.download_sender.send(msg::DownloadResponse {
            r#type: Some(msg::download_response::Type::Initated(msg::DownloadInitated {
              number_of_chunks: request.number_of_chunks,
              checksum: request.checksum.clone(),
              buffer_size: request.buffer_size,
              permissions: request.permissions.clone(),
            })),
          }) {
            eprintln!("Error sending download initiated to download sender: {:?}", err);
          }
        } else {
          eprintln!("No connection found for key {:?}", key);
        }
      }
      DownloadEvent::RequestFileChunk(key, offset) => {
        if let Some(connection) = self.connections.get_mut(&key) {
          if let Some(upload_sender) = &connection.upload_sender {
            println!("Sending chunk request for key {:?}", key);
            if let Err(err) = upload_sender.send(msg::UploadResponse {
              r#type: Some(msg::upload_response::Type::Request(msg::ChunkRequest { offset })),
            }) {
              eprintln!("Error sending chunk request to upload sender: {:?}", err);
            }
          } else {
            eprintln!("No upload sender found for key {:?}", key);
          }
        } else {
          eprintln!("No connection found for key {:?}", key);
        }
      }
      DownloadEvent::SendFileChunk(key, chunk) => {
        if let Some(connection) = self.connections.get_mut(&key) {
          println!("Sending chunk for key {:?}", key);
          if let Err(err) = connection.download_sender.send(msg::DownloadResponse {
            r#type: Some(msg::download_response::Type::Chunk(msg::SharedFileChunk {
              offset: chunk.offset,
              data: chunk.data,
            })),
          }) {
            eprintln!("Error sending chunk to download sender: {:?}", err);
          }
        } else {
          eprintln!("No connection found for key {:?}", key);
        }
      }
      DownloadEvent::DownloadComplete(key) => {
        if let Some(connection) = self.connections.remove(&key) {
          if let Some(upload_sender) = connection.upload_sender {
            println!("Sending upload complete for key {:?}", key);
            if let Err(err) = upload_sender.send(msg::UploadResponse {
              r#type: Some(msg::upload_response::Type::Complete(msg::UploadComplete {})),
            }) {
              eprintln!("Error sending upload complete to upload sender: {:?}", err);
            }
          } else {
            eprintln!("No upload sender found for key {:?}", key);
          }
        } else {
          eprintln!("No connection found for key {:?}", key);
        }
      }
      DownloadEvent::ApplicationClosing => {
        self.connections.clear();
      }
    }
  }
}
