use core::panic;
use std::collections::BTreeMap;

use crate::actors::device_map::DeviceMap;
use crate::common as ids;
use sinnergasm::protos as msg;

const MAXIMUM_BUFFER_SIZE: u64 = 16384;

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
  DownloadRequested(ids::WorkspaceName, msg::InitiateDownload),
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
            if let Some(sender) = device_map.devices.get(&device) {
              device_map.target = Some((device.clone(), sender.clone()));
            } else {
              println!("Targetted workspace device is not present {:?}", device);
            }
            panic!("Target update event should not be sent to workspace actor");
          }
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
      SubscriptionEvent::DownloadRequested(workspace_name, initiate_request) => {
        if let Some(device_map) = self.listeners.get_mut(&workspace_name) {
          if let Some(uploader) = device_map.devices.get(&initiate_request.upload_device) {
            if let Err(err) = uploader.send(msg::WorkspaceEvent {
              event_type: Some(msg::workspace_event::EventType::DownloadRequest(msg::UploadRequested {
                download_device: initiate_request.download_device.clone(),
                file_path: initiate_request.file_path.clone(),
                buffer_size: initiate_request
                  .buffer_size
                  .map(|x| std::cmp::max(x, MAXIMUM_BUFFER_SIZE)),
              })),
            }) {
              println!("Failed to send download request to uploader: {:?}", err);
            }
          } else {
            println!("No uploader device found for workspace: {:?}", workspace_name);
          }
        } else {
          println!("No workspace listeners for workspace: {:?}", workspace_name);
        }
      }
      SubscriptionEvent::ApplicationClosing => {
        self.listeners.clear();
      }
      SubscriptionEvent::TargetEvent(workspace_name, device_name, clipboard) => {
        self.handle_target_event(workspace_name, device_name, clipboard);
      }
    }
  }

  fn handle_target_event(&mut self, workspace_name: String, device_name: String, clipboard: Option<String>) {
    // TODO: clean this method up, DRY
    if let Some(device_map) = self.listeners.get_mut(&workspace_name) {
      if let Some((target, _)) = device_map.target.as_ref() {
        println!("Current target: {}, new target {}", target, device_name);
        // No change in target
        if target == &device_name {
          println!("Target device already targetted: {} in {}", device_name, workspace_name);
          return;
        }
        // there is existing target
        device_map.devices.retain(|device, sender| {
          sender
            .send(if device == target {
              get_target_message(TargetType::OldTarget, &device_name, &clipboard)
            } else if device == &device_name {
              get_target_message(TargetType::NewTarget, &device_name, &clipboard)
            } else {
              get_target_message(TargetType::Neither, &device_name, &clipboard)
            })
            .map_err(|err| {
              println!("Failed to send event to listener: {:?}", err);
              err
            })
            .is_ok()
        });
      } else {
        println!(
          "No existing target for workspace: {}. new target: {}",
          workspace_name, device_name
        );
        // No existing target
        device_map.devices.retain(|device, sender| {
          sender
            .send(if device == &device_name {
              get_target_message(TargetType::NewTarget, &device_name, &clipboard)
            } else {
              get_target_message(TargetType::Neither, &device_name, &clipboard)
            })
            .map_err(|err| {
              println!("Failed to send event to listener: {:?}", err);
              err
            })
            .is_ok()
        });
      }

      if let Some(sender) = device_map.devices.get(&device_name) {
        device_map.target = Some((device_name.clone(), sender.clone()));
      } else {
        println!("Targetted workspace device is not present {:?}", device_name);
      }
    } else {
      println!("Target: No simulation listeners for workspace: {}", workspace_name);
    }
  }
}

enum TargetType {
  NewTarget,
  OldTarget,
  Neither,
}

fn get_target_message(
  target_type: TargetType,
  device_name: &String,
  clipboard: &Option<String>,
) -> msg::WorkspaceEvent {
  msg::WorkspaceEvent {
    event_type: Some(match target_type {
      TargetType::NewTarget => msg::workspace_event::EventType::Targetted(msg::Targetted {
        clipboard: clipboard.clone(),
      }),
      TargetType::OldTarget => msg::workspace_event::EventType::Untargetted(msg::Untargetted {
        device: device_name.clone(),
      }),
      TargetType::Neither => msg::workspace_event::EventType::TargetUpdate(msg::TargetUpdate {
        device: device_name.clone(),
      }),
    }),
  }
}
