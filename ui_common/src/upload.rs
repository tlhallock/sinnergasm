use std::sync::Arc;

use crate::events;
use sha2::Digest;
use sha2::Sha256;
use sinnergasm::grpc_client::GrpcClient;
use sinnergasm::options::Options;
use sinnergasm::protos as msg;
use std::fs;
use std::io;
use std::io::prelude::*;
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc as tokio_mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

pub(crate) fn compute_hash(file_path: &std::path::Path) -> Result<String, anyhow::Error> {
  let mut hasher = Sha256::new();
  let mut file = fs::File::open(file_path)?;

  let bytes_written = io::copy(&mut file, &mut hasher)?;
  if bytes_written == 0 {
    eprintln!("hashed 0 bytes");
  }
  let hash_bytes = hasher.finalize();
  Ok(format!("{:x}", hash_bytes))
}

async fn upload_file(
  mut client: GrpcClient,
  request: msg::UploadRequested,
  options: Arc<Options>,
) -> Result<(), anyhow::Error> {
  let file_path = std::path::Path::new(&options.shared_folder).join(&request.relative_path);

  let checksum = compute_hash(&file_path)?;

  let mut file = std::fs::File::open(&file_path)?;
  let metadata = file.metadata()?;
  let permissions = format!("{:?}", metadata.permissions());

  let buffer_size = request.buffer_size.unwrap_or(4096);
  let number_of_chunks = metadata.len().div_ceil(buffer_size);

  let (sender, receiver) = tokio_mpsc::unbounded_channel();
  let receiver_stream = UnboundedReceiverStream::new(receiver);
  let mut stream = client.upload_file(receiver_stream).await?.into_inner();

  sender.send(msg::UploadRequest {
    r#type: Some(msg::upload_request::Type::Initiate(msg::InitiateUpload {
      workspace: options.workspace.clone(),
      download_device: request.download_device.clone(),
      upload_device: options.device.clone(),
      relative_path: request.relative_path.clone(),
      buffer_size,
      checksum,
      number_of_chunks,
      permissions,
    })),
  })?;

  while let Some(msg::UploadResponse {
    r#type: Some(event_type),
  }) = stream.message().await?
  {
    match event_type {
      msg::upload_response::Type::Request(msg::ChunkRequest { offset }) => {
        let byte_offset = offset * buffer_size;
        file.seek(std::io::SeekFrom::Start(byte_offset))?;
        let mut buf = vec![0; buffer_size as usize];
        if let Ok(size) = file.read(&mut buf) {
          if size == 0 {
            eprintln!("Read 0 bytes!");
          }
          sender.send(msg::UploadRequest {
            r#type: Some(msg::upload_request::Type::Chunk(msg::SharedFileChunk {
              offset,
              data: buf.into(),
            })),
          })?;
        } else {
          eprintln!("Unable to read file");
        }
      }
      msg::upload_response::Type::Complete(msg::UploadComplete {}) => {
        break;
      }
    }
  }

  anyhow::Ok(())
}

pub async fn listen_for_uploads(
  mut receiver: Receiver<events::AppEvent>,
  client: GrpcClient,
  options: Arc<Options>,
) -> Result<(), anyhow::Error> {
  loop {
    match receiver.recv().await? {
      events::AppEvent::Quit => {
        println!("Received quit event");
        return Ok(());
      }
      events::AppEvent::SubscriptionEvent(events::SubscriptionEvent::BeginUpload(request)) => {
        let client_clone = client.clone();
        let options_clone = options.clone();
        let _ = tokio::task::spawn(async move {
          if let Err(err) = upload_file(client_clone, request, options_clone).await {
            eprintln!("Error uploading file: {}", err);
          }
        });
      }
      _ => {}
    }
  }
}
