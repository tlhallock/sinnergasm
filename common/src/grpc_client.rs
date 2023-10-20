extern crate rdev;

use tonic::transport::Channel;
use tonic::Status;

use tokio::time::timeout;
use tokio::time::Duration;
use tonic::metadata::Ascii;
use tonic::service::Interceptor;
// pub mod display2;

use crate::protos::virtual_workspaces_client::VirtualWorkspacesClient;
use anyhow;
use tonic::metadata::MetadataValue;
// use tonic::tansport::Channel;
use crate::options::Options;

pub type GrpcClient = VirtualWorkspacesClient<tonic::codegen::InterceptedService<Channel, AuthorizationInterceptor>>;

#[derive(Clone)]
pub struct AuthorizationInterceptor {
  token: MetadataValue<Ascii>,
}

impl AuthorizationInterceptor {
  fn new(token: MetadataValue<Ascii>) -> Self {
    Self { token }
  }
}

impl Interceptor for AuthorizationInterceptor {
  fn call(&mut self, mut request: tonic::Request<()>) -> Result<tonic::Request<()>, Status> {
    request.metadata_mut().insert("authorization", self.token.clone());
    Ok::<_, Status>(request)
  }
}

// pub struct GrpcClient {
//   inner: VirtualWorkspacesClient<tonic::codegen::InterceptedService<Channel, AuthorizationInterceptor>>,
// }

pub async fn create_client(options: &Options) -> Result<GrpcClient, anyhow::Error> {
  let channel = Channel::from_shared(options.base_url.clone())?.concurrency_limit(options.concurrency_limit);
  let connect_future = channel.connect();
  let channel = timeout(Duration::from_secs(options.timeout), connect_future).await??;
  let interceptor = AuthorizationInterceptor::new(format!("Bearer {}", options.token).parse()?);
  let client = VirtualWorkspacesClient::with_interceptor(channel, interceptor);
  Ok(client)
}
