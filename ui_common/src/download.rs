use std::collections::HashSet;
use std::sync::Arc;

use sinnergasm::grpc_client::GrpcClient;
use sinnergasm::options::Options;
use sinnergasm::protos as msg;
use std::io;
use std::io::prelude::*;
use tokio::sync::mpsc as tokio_mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

#[derive(Debug)]
enum ProgressCheck {
  Complete,
  Requested(usize),
}

#[derive(Debug)]
struct DownloadChunkPool {
  to_download: Vec<u64>,
  downloading: Vec<(u64, std::time::SystemTime)>,
  downloaded: Vec<u64>,

  parallel_downloads: usize,
  sender: tokio_mpsc::UnboundedSender<msg::DownloadRequest>,
}

impl DownloadChunkPool {
  fn new(
    number_of_chunks: u64,
    parallel_downloads: usize,
    sender: tokio_mpsc::UnboundedSender<msg::DownloadRequest>,
  ) -> Self {
    Self {
      to_download: (0..number_of_chunks).collect(),
      downloading: vec![],
      downloaded: vec![],
      parallel_downloads,
      sender,
    }
  }

  fn set_completed(&mut self, offset: Option<u64>) -> Result<ProgressCheck, anyhow::Error> {
    if let Some(offset) = offset {
      if let Some(index) = self.downloading.iter().position(|&(x, _)| x == offset) {
        println!("Removing chunk {} from downloading", offset);
        self.downloading.remove(index);
        self.downloaded.push(offset);
      } else {
        panic!("Chunk {} was not being downloaded", offset);
      }
    }

    let mut num_requested = 0;
    let now = std::time::SystemTime::now();
    while self.downloading.len() < self.parallel_downloads && !self.to_download.is_empty() {
      let next = self.to_download.remove(0);
      self.downloading.push((next, now));

      self.sender.send(msg::DownloadRequest {
        r#type: Some(msg::download_request::Type::Request(msg::ChunkRequest {
          offset: next,
        })),
      })?;
      num_requested += 1;
    }

    println!("Download state: {:?}", self);

    if self.to_download.is_empty() && self.downloading.is_empty() {
      self.sender.send(msg::DownloadRequest {
        r#type: Some(msg::download_request::Type::Complete(msg::DownloadComplete {})),
      })?;
      Ok(ProgressCheck::Complete)
    } else {
      Ok(ProgressCheck::Requested(num_requested))
    }
  }
}

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
  let target_directory = target_location
    .parent()
    .expect(format!("Unable to determine parent directory: {:?}", &target_location).as_str());
  println!("Spawned thread: creating directory");
  std::fs::create_dir_all(target_directory)
    .expect(format!("Unable to create directory: {:?}", &target_directory).as_str());

  let mut file = std::fs::File::create(&target_location)?;

  println!("Spawned thread: created file");

  if let Some(msg::DownloadResponse {
    r#type:
      Some(msg::download_response::Type::Initated(msg::DownloadInitated {
        number_of_chunks,
        checksum: expected_checksum,
        buffer_size,
        permissions,
      })),
  }) = stream.message().await?
  {
    println!("Received download initiated message");
    let mut downloads = DownloadChunkPool::new(number_of_chunks, 1, sender);
    println!("initial state: {:?}", downloads);
    match downloads.set_completed(None) {
      Ok(ProgressCheck::Complete) => {
        eprintln!("Download complete after initial request!!");
      }
      Ok(ProgressCheck::Requested(num_requested)) => {
        println!("Requested initial chunks: {:?}", num_requested);
      }
      Err(err) => eprintln!("Unable to send download initial requests: {:?}", err),
    }
    println!("second state: {:?}", downloads);

    'outer: while let Some(msg::DownloadResponse {
      r#type: Some(event_type),
    }) = stream.message().await?
    {
      match event_type {
        msg::download_response::Type::Initated(_) => {
          panic!("Download should only be initiated once");
        }
        msg::download_response::Type::Chunk(msg::SharedFileChunk { offset, data }) => {
          println!("Received chunk {}", offset);

          file.seek(io::SeekFrom::Start(offset * buffer_size))?;
          file.write_all(&data)?;

          match downloads.set_completed(Some(offset)) {
            Ok(ProgressCheck::Complete) => {
              println!("Download complete");
              break 'outer;
            }
            Ok(ProgressCheck::Requested(num_requested)) => {
              println!("Requesting chunks: {:?}", num_requested);
            }
            Err(err) => eprintln!("Unable to send download requests: {:?}", err),
          }

          println!("Download state: {:?}", downloads);
        }
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
