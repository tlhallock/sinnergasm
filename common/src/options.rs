// pub const HOST: &str = "localhost";
// pub const HOST: &str = "54.177.251.247";
// pub const HOST: &str = "13.56.156.19";
// pub const HOST: &str =
//   "sinnergy-nlb-bdef3e305c57149f.elb.us-west-1.amazonaws.com";

use std::time::Duration;

pub const HOST: &str = "10.0.0.129";

pub const PORT: i64 = 50051;

pub fn read_token() -> String {
  std::fs::read_to_string("./keys/token.txt")
    .expect("Unable to read token.")
    .trim()
    .into()
}

#[derive(Clone)]
pub struct Options {
  pub base_url: String,
  pub token: String,
  pub workspace: String,
  pub device: String,
  pub timeout: u64,
  pub concurrency_limit: usize,
  pub controller_mouse_frequency: Duration,
  pub capacity: usize,
  pub shared_folder: String,
}

impl Options {
  pub fn new(device: String) -> Self {
    Self {
      base_url: format!("http://{}:{}", HOST, PORT).into(),
      token: read_token(),
      workspace: "The Workspace".into(),
      device,
      timeout: 5,
      concurrency_limit: 256,
      controller_mouse_frequency: Duration::from_millis(20),
      capacity: 256,
      shared_folder: "/work/ProjectsForFun/rust-synergy/seperate/upload_directory".into(),
    }
  }
}
