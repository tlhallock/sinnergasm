
use std::collections::BTreeMap;

use crate::common as ids;
use sinnergasm::protos as msg;


pub(crate) enum SubscriptionEvent {
  Subscribe(ids::WorkspaceName, ids::DeviceName, tokio::sync::mpsc::UnboundedSender<msg::WorkspaceEvent>),
  Unsubscribe(ids::WorkspaceName, ids::DeviceName),
  WorkspaceEvent(ids::WorkspaceName, msg::WorkspaceEvent),
  WorskpaceClosing(ids::WorkspaceName),
  ApplicationClosing,
}


#[derive(Debug, Default)]
pub(crate) struct WorkspaceActor {
  listeners: BTreeMap<ids::WorkspaceName, BTreeMap<ids::DeviceName, tokio::sync::mpsc::UnboundedSender<msg::WorkspaceEvent>>>,
}

impl WorkspaceActor
{
  pub(crate) fn receive(
    &mut self,
    event: SubscriptionEvent,
  ) {
    match event {
      SubscriptionEvent::Subscribe(
        workspace_name, device_name, sender
      ) => {
        self
          .listeners
          .entry(workspace_name)
          .or_insert_with(BTreeMap::new)
          .insert(device_name, sender);
      },
      SubscriptionEvent::Unsubscribe(workspace_id, device_id) => {
        if let Some(device_map) = self.listeners.get_mut(&workspace_id) {
          device_map.remove(&device_id);
          // Do this in the workspace actor...
          if device_map.is_empty() {
            self.listeners.remove(&workspace_id);
          }
        }
      },
      SubscriptionEvent::WorskpaceClosing(workspace_name) => {
        self.listeners.remove(&workspace_name);
      },
      SubscriptionEvent::WorkspaceEvent(workspace_name, event) => {
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
