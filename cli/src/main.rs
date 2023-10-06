extern crate clap;
use clap::{App, Arg, SubCommand};

use sinnergasm::protos as msg;
use sinnergasm::protos::virtual_workspaces_client::VirtualWorkspacesClient;
use sinnergasm::SECRET_TOKEN;
use tokio_stream::{self, StreamExt};
use tonic::metadata::MetadataValue;
use tonic::transport::Channel;
use tonic::Request;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let base_url = "http://localhost:50051";
  let channel = Channel::from_static(base_url)
    .concurrency_limit(256)
    .connect()
    .await?;

  let token: MetadataValue<_> = format!("Bearer {SECRET_TOKEN}",).parse()?;
  let mut client = VirtualWorkspacesClient::with_interceptor(
    channel,
    move |mut req: Request<()>| {
      req.metadata_mut().insert("authorization", token.clone());
      Ok(req)
    },
  );

  Ok(())
}
