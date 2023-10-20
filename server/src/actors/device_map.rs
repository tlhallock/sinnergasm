use std::collections::BTreeMap;

use crate::common as ids;

use std::fmt::Debug;
use std::fmt::Formatter;
use tokio::sync::mpsc as tokio_mpsc;

// #[derive(Default)]
pub(crate) struct DeviceMap<T> {
  pub(crate) target: Option<(ids::DeviceName, tokio_mpsc::UnboundedSender<T>)>,
  pub(crate) devices: BTreeMap<ids::DeviceName, tokio_mpsc::UnboundedSender<T>>,
}

impl<T> DeviceMap<T> {
  pub(crate) fn remove(&mut self, device: &ids::DeviceName) {
    self.devices.remove(device);
  }
  pub(crate) fn is_empty(&self) -> bool {
    self.devices.is_empty()
  }
}

impl<T> Default for DeviceMap<T> {
  fn default() -> Self {
    Self {
      target: None,
      devices: BTreeMap::new(),
    }
  }
}

impl<T> Debug for DeviceMap<T> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("DeviceMap")
      .field("target", &self.target.as_ref().map(|(name, _)| name))
      .field("devices", &self.devices.keys())
      .finish()
  }
}
