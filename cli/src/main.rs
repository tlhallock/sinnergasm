
use tonic::Request;
use std::sync::Arc;
use sinnergasm::options::Options;
use sinnergasm::grpc_client::create_client;
use sinnergasm::protos as msg;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let options = Arc::new(Options::new("desktop".into()));
  let mut client = create_client(&options).await?;

  println!("Sending close workspace request");
  client.close_workspace(msg::CloseRequest {
    workspace: options.workspace.clone(),
  }).await?;

//   {
//     let request = msg::ListRequest {};
//     let response = client.list_workspaces(request).await;
//     if let Ok(response) = response {
//       println!("Response: {:?}", response);
//     }
//   }

    Ok(())
}