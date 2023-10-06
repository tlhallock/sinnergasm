
use std::collections::BTreeMap;
use crate::common::WorkspaceId;
use crate::events::WorkspaceEvent;


#[derive(Debug)]
pub(crate) struct WorkspaceActor {
    listeners: BTreeMap<WorkspaceId, tokio::sync::mpsc::UnboundedSender<WorkspaceEvent>>,
}


impl<T> WorkspaceActor<T> {
    pub(crate) fn new() -> Self {
        Self {
            listeners: BTreeMap::new(),
        }
    }

    pub(crate) fn send_event(
        &mut self,
        workspace_id: WorkspaceId,
        event: T,
    ) -> Result<(), tokio::sync::mpsc::error::SendError<T>> {
        if let Some(listener) = self.listeners.get_mut(&workspace_id) {
            listener.send(event)
        } else {
          panic!("There is still a sender, although there is no listener for workspace {}", workspace_id);
        }
    }

    pub(crate) fn subscribe(
        &mut self,
        workspace_id: WorkspaceId,
        listener: tokio::sync::mpsc::UnboundedSender<T>,
    ) {
        self.listeners.insert(workspace_id, listener);
    }

    pub(crate) fn unsubscribe(&mut self, workspace_id: WorkspaceId) {
        self.listeners.remove(&workspace_id);
    }
}
