use std::sync::Arc;

use sha2::Digest;
use sha2::Sha256;
use sinnergasm::grpc_client::GrpcClient;
use sinnergasm::options::Options;
use sinnergasm::protos as msg;
use std::fs;
use std::io;
use std::io::prelude::*;
use tokio::sync::mpsc as tokio_mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

pub async fn download_file(
  mut client: GrpcClient,
  upload_device: &String,
  shared_file: &msg::SharedFile,
  options: Arc<Options>,
) -> Result<(), anyhow::Error> {
  let (sender, receiver) = tokio_mpsc::unbounded_channel();
  let receiver_stream = UnboundedReceiverStream::new(receiver);
  let mut stream = client.download_file(receiver_stream).await?.into_inner();
  let target_location = shared_file.file_path.clone();

  sender.send(msg::DownloadRequest {
    r#type: Some(msg::download_request::Type::Initiate(msg::InitiateDownload {
      workspace: options.workspace.clone(),
      download_device: options.device.clone(),
      upload_device: upload_device.clone(),
      file_path: shared_file.file_path.clone(),
      buffer_size: None,
    })),
  })?;

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
