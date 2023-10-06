use std::collections::BTreeMap;

use crate::common as ids;

pub(crate) enum SubscriptionEvent<T> {
  ListenerCreated(ids::WorkspaceName, ids::DeviceName, tokio::sync::mpsc::UnboundedSender<T>),
  ListenerRemoved(ids::WorkspaceName, ids::DeviceName),
  WorskpaceClosed(ids::WorkspaceName),
  Event(ids::WorkspaceName, T),
  ApplicationClosing,
}


#[derive(Debug)]
pub(crate) struct Actor<T> {
  listeners: BTreeMap<
    ids::WorkspaceName,
    BTreeMap<ids::DeviceName, tokio::sync::mpsc::UnboundedSender<T>>,
  >,
}

impl<T> Actor<T> 
where
  T: Clone + Send + 'static,
  {
  pub(crate) fn new() -> Self {
    Self {
      listeners: BTreeMap::new(),
    }
  }

  pub(crate) fn receive(
    &mut self,
    event: SubscriptionEvent<T>,
  ) {
    match event {
      SubscriptionEvent::ListenerCreated(
        workspace_name, device_name, sender
      ) => {
        self
          .listeners
          .entry(workspace_name)
          .or_insert_with(BTreeMap::new)
          .insert(device_name, sender);
      },
      SubscriptionEvent::ListenerRemoved(workspace_id, device_id) => {
        if let Some(device_map) = self.listeners.get_mut(&workspace_id) {
          device_map.remove(&device_id);
          // Do this in the workspace actor...
          if device_map.is_empty() {
            self.listeners.remove(&workspace_id);
          }
        }
      },
      SubscriptionEvent::WorskpaceClosed(workspace_name) => {
        self.listeners.remove(&workspace_name);
      },
      SubscriptionEvent::Event(workspace_name, event) => {
        if let Some(device_map) = self.listeners.get_mut(&workspace_name) {
          device_map.retain(|_, listener| {
            if let Err(err) = listener.send(event.clone()) {
              println!("Failed to send event to listener: {:?}", err);
              return false;
            }
            return true;
          });
        }
      },
      SubscriptionEvent::ApplicationClosing => {
        self.listeners.clear();
      },
    }
  }
}
