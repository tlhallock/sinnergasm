use std::sync::Arc;

use sinnergasm::grpc_client::GrpcClient;
use sinnergasm::options::Options;
use sinnergasm::protos as msg;
use std::io;
use std::io::prelude::*;
use tokio::sync::mpsc as tokio_mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

pub async fn spawn_download_task(
  client: GrpcClient,
  upload_device: String,
  shared_file: msg::SharedFile,
  options: Arc<Options>,
) -> tokio::task::JoinHandle<anyhow::Result<()>> {
  println!("Spawning download task");
  let task = tokio::spawn(async move {
    println!("Inside download task");
    if let Err(err) = download_file(client, upload_device, shared_file, options).await {
      eprintln!("Error downloading file: {}", err);
    }
    println!("Download task finished");
    Ok(())
  });
  return task;
}

async fn download_file(
  mut client: GrpcClient,
  upload_device: String,
  shared_file: msg::SharedFile,
  options: Arc<Options>,
) -> Result<(), anyhow::Error> {
  println!("Inside download_file");
  let (sender, receiver) = tokio_mpsc::unbounded_channel();
  println!("Spawned thread: creating receiver stream");
  let receiver_stream = UnboundedReceiverStream::new(receiver);
  println!("Spawned thread: sending download request");
  sender.send(msg::DownloadRequest {
    r#type: Some(msg::download_request::Type::Initiate(msg::InitiateDownload {
      workspace: options.workspace.clone(),
      download_device: options.device.clone(),
      upload_device: upload_device.clone(),
      relative_path: shared_file.relative_path.clone(),
      buffer_size: None,
    })),
  })?;
  println!("Spawned thread: creating download stream");
  let mut stream = client.download_file(receiver_stream).await?.into_inner();
  println!("Spawned thread: joining paths");
  let target_location = std::path::Path::new(&options.shared_folder).join(&shared_file.relative_path);

  let mut file = std::fs::File::create(&target_location)?;

  if let Some(msg::DownloadResponse {
    r#type:
      Some(msg::download_response::Type::Initated(msg::DownloadInitated {
        number_of_chunks,
        checksum: expected_checksum,
        buffer_size,
      })),
  }) = stream.message().await?
  {
    let mut chunks_received = vec![false; number_of_chunks as usize];
    // let mut chunks_requested =

    while let Some(msg::DownloadResponse {
      r#type: Some(event_type),
    }) = stream.message().await?
    {
      match event_type {
        msg::download_response::Type::Initated(_) => {
          panic!("Download should only be initiated once");
        }
        msg::download_response::Type::Chunk(msg::SharedFileChunk { offset, data }) => {
          println!("Received chunk {}", offset);
          chunks_received[offset as usize] = true;

          file.seek(io::SeekFrom::Start(offset as u64 * buffer_size as u64))?;
          file.write_all(&data)?;

          let next_offset = offset + 1;
          if next_offset >= number_of_chunks {
            break;
          }

          sender.send(msg::DownloadRequest {
            r#type: Some(msg::download_request::Type::Request(msg::ChunkRequest {
              offset: next_offset,
            })),
          })?;
        }
      }
      if chunks_received.iter().all(|x| *x) {
        break;
      }
    }

    let actual_checksum = crate::upload::compute_hash(&target_location)?;
    println!(
      "Expected checksum {}. Found checksum {}",
      expected_checksum, actual_checksum
    );
  } else {
    panic!("First message should be the upload has been initiated.");
  }

  anyhow::Ok(())
}
