extern crate rdev;

pub mod display;
pub mod events;
pub mod handler;
pub mod listener;
pub mod options;
pub mod prison;
use tokio::time::timeout;
use tokio::time::Duration;
// pub mod display2;

use std::sync::mpsc;

use crate::display::launch_display;
use crate::handler::forward_events;
use crate::listener::listen_to_keyboard_and_mouse;
use anyhow;
use sinnergasm::options::Options;
use sinnergasm::protos as msg;
use sinnergasm::protos::virtual_workspaces_client::VirtualWorkspacesClient;
use tokio_stream;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tonic::metadata::MetadataValue;
use tonic::transport::Channel;
use tonic::{Request, Status};


#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let options = Options::new("desktop".into());
  let channel = Channel::from_shared(options.base_url.clone())?
    .concurrency_limit(options.concurrency_limit);
  let connect_future = channel.connect();
  let channel =
    timeout(Duration::from_secs(options.timeout), connect_future).await??;
  let token: MetadataValue<_> = format!("Bearer {}", options.token).parse()?;
  let mut client = VirtualWorkspacesClient::with_interceptor(
    channel,
    move |mut req: Request<()>| {
      req.metadata_mut().insert("authorization", token.clone());
      Ok::<_, Status>(req)
    },
  );


  let other_devices = {
    let request = msg::GetRequest { name: options.workspace.clone(), };
    let workspace = client.get_workspace(request).await?.into_inner();
    println!("Connecting to workspace: {:?}", workspace);
    workspace.devices.iter().filter(|device| device.name != options.device).cloned().collect::<Vec<_>>()
  };

  let (app_snd, app_rcv) = mpsc::channel();
  let (control_snd, control_rcv) =
    tokio::sync::mpsc::unbounded_channel::<msg::ControlRequest>();
  let network_task = tokio::task::spawn(async move {
    client
      .control_workspace(UnboundedReceiverStream::new(control_rcv))
      .await
  });

  let subscription_request = msg::WorkspaceSubscriptionRequest {
    workspace: options.workspace.clone(),
    device: options.device.clone(),
  };
  let targetted_sender = app_snd.clone();
  let current_device = options.device.clone();
  let network_task = tokio::task::spawn(async move {
    let subscription = client
      .subscribe_to_workspace(subscription_request)
      .await?.into_inner();
    while let Some(message) = subscription.message().await? {
      if let Some(message) = message.event_type {
        match message {
          msg::workspace_event::EventType::TargetUpdate(msg::TargetUpdate { device }) => {
            if device != current_device {
              continue;
            }
            targetted_sender.send(
              events::ControlEvent::WeBeTargetted
            ).expect(
              "Unable to send targetted event"
            );
          }
          _ => {},
        }
      }
    }
    anyhow::Ok(())
  });

  let key_sender = app_snd.clone();
  let _ =
    tokio::task::spawn(async move { listen_to_keyboard_and_mouse(key_sender) });

  let forward_task =
    tokio::task::spawn(
      async move { forward_events(app_rcv, control_snd).await },
    );


  launch_display(app_snd, other_devices)?;
  // TODO: cleanly close the connections...
  panic!("Display closed");

  forward_task.await??;
  network_task.await??;

  anyhow::Ok(())
}
