extern crate rdev;

pub mod display;
pub mod events;
pub mod handler;
pub mod listener;
pub mod prison;
// pub mod display2;

use std::sync::mpsc;

use crate::display::launch_display;
use crate::handler::forward_events;
use crate::listener::listen_to_keyboard_and_mouse;
use anyhow;
use sinnergasm::protos as msg;
use sinnergasm::protos::virtual_workspaces_client::VirtualWorkspacesClient;
use sinnergasm::SECRET_TOKEN;
use tokio_stream;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tonic::metadata::MetadataValue;
use tonic::transport::Channel;
use tonic::{Request, Status};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let (app_snd, app_rcv) = mpsc::channel();
  let (control_snd, control_rcv) =
    tokio::sync::mpsc::unbounded_channel::<msg::ControlRequest>();
  // let (control_snd, mut control_rcv) =
  //   tokio::sync::mpsc::channel::<msg::ControlRequest>(10);

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
      Ok::<_, Status>(req)
    },
  );

  let network_task = tokio::task::spawn(async move {
    // println!("Starting network task");
    // while let Some(w) = control_rcv.recv().await {
    //   println!("Got workspace: {:?}", w);
    // }
    // println!("Dropping the receiver");
    // Ok(())
    client
      .control_workspace(UnboundedReceiverStream::new(control_rcv))
      .await
  });

  let key_sender = app_snd.clone();
  let _ =
    tokio::task::spawn(async move { listen_to_keyboard_and_mouse(key_sender) });

  let display_sender = app_snd.clone();
  let display_task =
    tokio::task::spawn(async move { launch_display(display_sender) });

  let forward_task =
    tokio::task::spawn(
      async move { forward_events(app_rcv, control_snd).await },
    );

  let (r1, r2, r3) = tokio::join!(display_task, forward_task, network_task);
  r1??;
  r2??;
  r3??;
  anyhow::Ok(())
}
