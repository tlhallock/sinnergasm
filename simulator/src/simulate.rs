use std::time::Duration;

use anyhow;
use rdev::simulate;
use sinnergasm::options::Options;
use sinnergasm::protos as msg;
use sinnergasm::protos::virtual_workspaces_client::VirtualWorkspacesClient;
use tokio::time::timeout;
use tokio_stream;
use tonic::metadata::MetadataValue;
use tonic::transport::Channel;
use tonic::{Request, Status};
use sinnergasm::errors::RDevError;

// let cert = std::fs::read_to_string("ca.pem")?;
// .tls_config(ClientTlsConfig::new()
//     .ca_certificate(Certificate::from_pem(&cert))
//     .domain_name("example.com".to_string()))?
// .timeout(Duration::from_secs(5))
// .rate_limit(5, Duration::from_secs(1))


enum SimulatorEvent {
  LocalMouseChanged(f64, f64),
  SimulateEvent(msg::SimulationEvent),
}

fn simulate_input_event((mouse_x, mouse_y): (f64, f64), event: msg::user_input_event::Type) -> Result<(), rdev::SimulateError> {
  match event {
    msg::user_input_event::Type::MouseMove(msg::MouseMoveEvent { delta_x, delta_y }) => {
      simulate(&rdev::EventType::MouseMove {
        x: mouse_x + delta_x,
        y: mouse_y + delta_y,
      })
    }
    msg::user_input_event::Type::MouseButton(msg::MouseButtonEvent {}) => {
      Ok(())
    }
    msg::user_input_event::Type::Keyboard(msg::KeyboardEvent {}) => {
      Ok(())
    }
    msg::user_input_event::Type::Wheel(msg::WheelEvent { dx, dy}) => {
      simulate(&rdev::EventType::Wheel {
        delta_x: dx.into(), delta_y: dy.into()
      })
    }
  }
}


async fn simulate_receiver(
  mut receiver: tokio::sync::mpsc::UnboundedReceiver<SimulatorEvent>,
) -> Result<(), rdev::SimulateError> {
  let mut current_mouse_position = None;
  while let Some(event) = receiver.recv().await {
    match event {
      SimulatorEvent::LocalMouseChanged(x, y) => {
        current_mouse_position = Some((x, y));
      },
      SimulatorEvent::SimulateEvent(event) => {
        if let Some((mouse_x, mouse_y)) = current_mouse_position {
        if let Some(event) = event.input_event {
          if let Some(event) = event.r#type {
            // Fail on first error?
            simulate_input_event((mouse_x, mouse_y), event)?
          }
        }
      } else {
        println!("No mouse event yet, we do not know the current location of the mouse.");
      }
      }
    }
  }
  Ok(())
}



pub(crate) fn listen_to_mouse(
  sender: tokio::sync::mpsc::UnboundedSender<SimulatorEvent>,
) -> Result<(), RDevError> {
  rdev::listen(move |event| {
    if let rdev::EventType::MouseMove { x, y } = event.event_type {
      if let Err(e) = sender.send(SimulatorEvent::LocalMouseChanged(x, y)) {
        eprintln!("Error: {:?}", e);
      }
    }
  })?;
  Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let options = Options::new("laptop".into());
  let channel = Channel::from_shared(options.base_url.clone())?
    .concurrency_limit(options.concurrency_limit);
  let connect_future = channel.connect();
  let channel = timeout(Duration::from_secs(options.timeout), connect_future).await??;
  let token: MetadataValue<_> = format!("Bearer {}", options.token).parse()?;
  let mut client = VirtualWorkspacesClient::with_interceptor(
    channel,
    move |mut req: Request<()>| {
      req.metadata_mut().insert("authorization", token.clone());
      Ok::<_, Status>(req)
    },
  );

  {
    let request = msg::ListRequest {};
    let response = client.list_workspaces(request).await;
    if let Ok(response) = response {
      println!("Response: {:?}", response);
    }
  }

  let (sender, receiver) = tokio::sync::mpsc::unbounded_channel::<SimulatorEvent>();

  let key_sender = sender.clone();
  let _ = tokio::task::spawn(async move { listen_to_mouse(key_sender) });

  let relay_task = tokio::task::spawn(async move {
    let request = msg::SimulateRequest {
      workspace: "The Workspace".into(),
      device: "".into(),
    };
    let response = client.simulate_workspace(request).await?;
    let mut stream = response.into_inner();
    while let Ok(event) = stream.message().await {
      match event {
        Some(event) => sender.send(SimulatorEvent::SimulateEvent(event))?,
        None => break,
      }
    }
    anyhow::Ok(())
  });

  let simulate_task = tokio::task::spawn(async move {
    simulate_receiver(receiver).await?;
    anyhow::Ok(())
  });

  simulate_task.await??;

  if let Err(err) = relay_task.await {
    eprintln!("Error: {}", err);
  }

  Ok(())
}
