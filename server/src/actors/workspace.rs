use std::collections::BTreeMap;

use crate::actors::device_map::DeviceMap;
use crate::common as ids;
use sinnergasm::protos as msg;

pub(crate) enum SubscriptionEvent {
  Subscribe(
    ids::WorkspaceName,
    ids::DeviceName,
    tokio::sync::mpsc::UnboundedSender<msg::WorkspaceEvent>,
  ),
  Unsubscribe(ids::WorkspaceName, ids::DeviceName),
  WorkspaceEvent(ids::WorkspaceName, msg::WorkspaceEvent),
  WorskpaceClosing(ids::WorkspaceName),
  ApplicationClosing,
  TargetEvent(ids::WorkspaceName, ids::DeviceName, Option<String>),
}

#[derive(Debug, Default)]
pub(crate) struct WorkspaceActor {
  listeners: BTreeMap<ids::WorkspaceName, DeviceMap<msg::WorkspaceEvent>>,
}

impl WorkspaceActor {
  pub(crate) fn receive(&mut self, event: SubscriptionEvent) {
    match event {
      SubscriptionEvent::Subscribe(workspace_name, device_name, sender) => {
        self
          .listeners
          .entry(workspace_name)
          .or_insert_with(DeviceMap::default)
          .devices
          .insert(device_name, sender);
      }
      SubscriptionEvent::Unsubscribe(workspace_id, device_id) => {
        if let Some(device_map) = self.listeners.get_mut(&workspace_id) {
          device_map.remove(&device_id);
          // Do this in the workspace actor...
          if device_map.is_empty() {
            self.listeners.remove(&workspace_id);
          }
        }
      }
      SubscriptionEvent::WorskpaceClosing(workspace_name) => {
        self.listeners.remove(&workspace_name);
      }
      SubscriptionEvent::WorkspaceEvent(workspace_name, event) => {
        if let Some(device_map) = self.listeners.get_mut(&workspace_name) {
          if let Some(msg::workspace_event::EventType::TargetUpdate(msg::TargetUpdate { device })) =
            event.clone().event_type
          {
            panic!("This shouldn't happen");
          }
          // if let Some(msg::workspace_event::EventType::TargetUpdate(
          //   msg::TargetUpdate { device },
          // )) = event.clone().event_type
          // {
          //   if let Some(device_sender) = device_map.devices.get_mut(&device) {
          //     if let Err(err) = device_sender.send(msg::WorkspaceEvent {
          //       event_type: Some(msg::workspace_event::EventType::Targetted(
          //         msg::Targetted { clipboard: None },
          //       )),
          //     }) {
          //       println!("Failed to send event to listener: {:?}", err);
          //     }
          //   } else {
          //     println!("Targetted device is not listening {:?}", device);
          //   }
          // }
          device_map.devices.retain(|_, listener| {
            if let Err(err) = listener.send(event.clone()) {
              println!("Failed to send event to listener: {:?}", err);
              return false;
            }
            return true;
          });
        } else {
          println!("No workspace listeners for workspace: {:?}", workspace_name);
        }
      }
      SubscriptionEvent::ApplicationClosing => {
        self.listeners.clear();
      }
      SubscriptionEvent::TargetEvent(workspace_name, device_name, clipboard) => {
        // TODO: DRY
        if let Some(device_map) = self.listeners.get_mut(&workspace_name) {
          if let Some((target, _)) = device_map.target.as_ref() {
            // No change in target
            if target == &device_name {
              println!("Target device already targetted: {} in {}", device_name, workspace_name);
              return;
            }
            // there is existing target
            device_map.devices.retain(|device, sender| {
              if device == target {
                if let Err(err) = sender.send(msg::WorkspaceEvent {
                  event_type: Some(msg::workspace_event::EventType::Untargetted(msg::Untargetted {
                    device: device_name.clone(),
                  })),
                }) {
                  println!("Failed to send event to listener: {:?}", err);
                  return false;
                } else {
                  return true;
                }
              } else if device == &device_name {
                if let Err(err) = sender.send(msg::WorkspaceEvent {
                  event_type: Some(msg::workspace_event::EventType::Targetted(msg::Targetted {
                    clipboard: clipboard.clone(),
                  })),
                }) {
                  println!("Failed to send event to listener: {:?}", err);
                  return false;
                } else {
                  return true;
                }
              } else {
                if let Err(err) = sender.send(msg::WorkspaceEvent {
                  event_type: Some(msg::workspace_event::EventType::TargetUpdate(msg::TargetUpdate {
                    device: device_name.clone(),
                  })),
                }) {
                  println!("Failed to send event to listener: {:?}", err);
                  return false;
                } else {
                  return true;
                }
              }
            });
          } else {
            // No existing target
            device_map.devices.retain(|device, sender| {
              if device == &device_name {
                if let Err(err) = sender.send(msg::WorkspaceEvent {
                  event_type: Some(msg::workspace_event::EventType::Targetted(msg::Targetted {
                    clipboard: clipboard.clone(),
                  })),
                }) {
                  println!("Failed to send event to listener: {:?}", err);
                  return false;
                } else {
                  return true;
                }
              } else {
                if let Err(err) = sender.send(msg::WorkspaceEvent {
                  event_type: Some(msg::workspace_event::EventType::TargetUpdate(msg::TargetUpdate {
                    device: device_name.clone(),
                  })),
                }) {
                  println!("Failed to send event to listener: {:?}", err);
                  return false;
                } else {
                  return true;
                }
              }
            });
          }
        } else {
          println!("Target: No simulation listeners for workspace: {}", workspace_name);
        }
        // if let Some(current_target) = sel
      }
    }
  }
}
