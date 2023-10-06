use crate::common::DeviceId;
use crate::common::WorkspaceId;

#[derive(Clone, Debug)]
pub(crate) enum WorkspaceSubscriptionEvent {
  SetTarget(WorkspaceId, DeviceId),
}

#[derive(Clone, Debug)]
pub(crate) enum ReplicationEvent {
  NewTarget(WorkspaceId, DeviceId),
}

#[derive(Debug)]
pub(crate) enum WorkspaceEvent {
  CreateWorkspace,
  DeleteWorkspace,
  ConfigureWorkspace,
}
