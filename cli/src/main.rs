use tonic::Request;
use tonic_health::proto::health_client::HealthClient;
use tonic_health::proto::HealthCheckRequest;
use sinergasm::protos::health::health_server::HealthServer;
use sinnergasm::

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let options = Options::new("cli".into());
    let channel = tonic::transport::Channel::from_static("http://[::1]:50051")
        .connect()
        .await?;

    let mut client = HealthClient::new(channel);

    let request = Request::new(HealthCheckRequest {
        service: "".into(), // Specify the service name if needed
    });

    let response = client.check(request).await?;

    println!("Health check response: {:?}", response);

    Ok(())
}