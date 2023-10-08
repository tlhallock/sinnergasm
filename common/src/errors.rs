use rdev;

#[derive(Debug)]
pub struct RDevError {
  inner: rdev::ListenError,
}

impl From<rdev::ListenError> for RDevError {
  fn from(error: rdev::ListenError) -> Self {
    RDevError { inner: error }
  }
}

impl std::fmt::Display for RDevError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "CustomError: {:?}", self.inner)
  }
}

impl std::error::Error for RDevError {}
