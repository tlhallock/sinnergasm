use crate::actors::device_map::DeviceMap;
use std::collections::BTreeMap;
use std::fmt::Debug;

use crate::common as ids;
use sinnergasm::protos as msg;

pub(crate) enum SimulationEvent {
  AddSimulator(
    ids::WorkspaceName,
    ids::DeviceName,
    tokio::sync::mpsc::UnboundedSender<msg::SimulationEvent>,
  ),
  RemoveSimulator(ids::WorkspaceName, ids::DeviceName),
  TargetEvent(ids::WorkspaceName, ids::DeviceName),
  SimulationEvent(ids::WorkspaceName, msg::SimulationEvent),
  WorkspaceClosing(ids::WorkspaceName),
  ApplicationClosing,
}

#[derive(Debug, Default)]
pub(crate) struct SimulationActor {
  listeners: BTreeMap<ids::WorkspaceName, DeviceMap<msg::SimulationEvent>>,
}

impl SimulationActor {
  pub(crate) fn receive(&mut self, event: SimulationEvent) {
    match event {
      SimulationEvent::AddSimulator(workspace_name, device_name, sender) => {
        let device_map = self.listeners.entry(workspace_name).or_insert_with(DeviceMap::default);
        // device_map.target = Some((device_name.clone(), sender.clone()));
        device_map.devices.insert(device_name, sender);
        println!("Added simulator for workspace");
      }
      SimulationEvent::RemoveSimulator(workspace_name, device_name) => {
        if let Some(device_map) = self.listeners.get_mut(&workspace_name) {
          if let Some((target, _)) = device_map.target.as_ref() {
            if target == &device_name {
              device_map.target = None;
            }
          }
          device_map.devices.remove(&device_name);
          if device_map.devices.is_empty() {
            self.listeners.remove(&workspace_name);
          }
        }
      }
      SimulationEvent::TargetEvent(workspace_name, device_name) => {
        if let Some(device_map) = self.listeners.get_mut(&workspace_name) {
          if let Some(sender) = device_map.devices.get(&device_name) {
            device_map.target = Some((device_name, sender.clone()));
          } else {
            println!(
              "Target: new simulation target not found: {} in {}. Maybe it is the controller?",
              device_name, workspace_name
            );
          }
        } else {
          println!("Target: No simulation listeners for workspace: {}", workspace_name);
        }
      }
      SimulationEvent::SimulationEvent(workspace_name, event) => {
        if let Some(device_map) = self.listeners.get_mut(&workspace_name) {
          if let Some((_, sender)) = device_map.target.as_ref() {
            if let Err(err) = sender.send(event.clone()) {
              println!("Failed to send event to listener: {:?}", err);
            }
          } else {
            println!("No target for workspace");
          }
        } else {
          println!("No simulation listeners for workspace: {}", workspace_name);
        }
      }
      SimulationEvent::WorkspaceClosing(workspace_name) => {
        self.listeners.remove(&workspace_name);
      }
      SimulationEvent::ApplicationClosing => {
        self.listeners.clear();
      }
    }
  }
}
