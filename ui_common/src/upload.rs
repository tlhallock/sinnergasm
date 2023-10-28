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

fn div_ceil(a: u64, b: u64) -> u64 {
  // Assuming a + b doesn't overflow
  (a + b - 1) / b
}

async fn upload_file(
  mut client: GrpcClient,
  request: msg::UploadRequested,
  options: Arc<Options>,
) -> Result<(), anyhow::Error> {
  println!("Uploading file {:?}", request);
  let file_path = std::path::Path::new(&options.shared_folder).join(&request.relative_path);

  println!("Computing hash of file");
  let checksum = compute_hash(&file_path)?;
  println!("Hash of file is {}", checksum);
  let mut file = std::fs::File::open(&file_path)?;
  println!("Opened file {:?}", file_path);
  let metadata = file.metadata()?;
  println!("File metadata: {:?}", metadata);
  let permissions = metadata.permissions();
  println!("Ignoring file permissions: {:?}", permissions);

  let buffer_size = request.buffer_size.unwrap_or(4096);
  let number_of_chunks = div_ceil(metadata.len(), buffer_size);
  println!(
    "File size: {}, buffer size: {}, number of chunks: {}",
    metadata.len(),
    buffer_size,
    number_of_chunks
  );
  println!("Creating channel");
  let (sender, receiver) = tokio_mpsc::unbounded_channel();
  println!("Creating receiver stream");
  let receiver_stream = UnboundedReceiverStream::new(receiver);
  println!("Sending upload request");
  sender.send(msg::UploadRequest {
    r#type: Some(msg::upload_request::Type::Initiate(msg::InitiateUpload {
      workspace: options.workspace.clone(),
      download_device: request.download_device.clone(),
      upload_device: options.device.clone(),
      relative_path: request.relative_path.clone(),
      buffer_size,
      checksum,
      number_of_chunks,
      permissions: None,
    })),
  })?;

  println!("Awaiting upload response");
  let mut stream = client.upload_file(receiver_stream).await?.into_inner();

  println!("Sending chunks");
  while let Some(msg::UploadResponse {
    r#type: Some(event_type),
  }) = stream.message().await?
  {
    println!("Received upload response: {:?}", event_type);
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
        println!("Received request to upload {:?}", request);
        let client_clone = client.clone();
        let options_clone = options.clone();
        let task = tokio::task::spawn(async move {
          println!("within spawn: Uploading file");
          if let Err(err) = upload_file(client_clone, request, options_clone).await {
            eprintln!("Error uploading file: {}", err);
          }
        });
        task.await?;
      }
      _ => {}
    }
  }
}
