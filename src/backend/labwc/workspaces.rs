use super::LabwcBackend;
use crate::protocol::WorkspaceInfo;
use wayland_client::{
    Connection, Dispatch, Proxy, QueueHandle,
    protocol::wl_registry::{self, WlRegistry},
};
use wayland_client::{WEnum, event_created_child};
use wayland_protocols::ext::workspace::v1::client::{
    ext_workspace_group_handle_v1::ExtWorkspaceGroupHandleV1,
    ext_workspace_handle_v1::ExtWorkspaceHandleV1, ext_workspace_manager_v1::ExtWorkspaceManagerV1,
};

struct WsState {
    manager: Option<ExtWorkspaceManagerV1>,
    workspaces: Vec<WsData>,
    done: bool,
}

struct WsData {
    name: String,
    active: bool,
    handle: Option<ExtWorkspaceHandleV1>,
}

impl Dispatch<WlRegistry, ()> for WsState {
    fn event(
        state: &mut Self,
        registry: &WlRegistry,
        event: <WlRegistry as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
            && interface == "ext_workspace_manager_v1"
        {
            state.manager = Some(registry.bind::<ExtWorkspaceManagerV1, (), Self>(
                name,
                version.min(1),
                qh,
                (),
            ));
        }
    }
}

impl Dispatch<ExtWorkspaceManagerV1, ()> for WsState {
    event_created_child!(WsState, ExtWorkspaceManagerV1, [
        0 => (ExtWorkspaceGroupHandleV1, ()),
        1 => (ExtWorkspaceHandleV1, ())
    ]);

    fn event(
        state: &mut Self,
        _proxy: &ExtWorkspaceManagerV1,
        event: <ExtWorkspaceManagerV1 as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        use wayland_protocols::ext::workspace::v1::client::ext_workspace_manager_v1::Event;
        match event {
            Event::Workspace { workspace } => {
                state.workspaces.push(WsData {
                    name: String::new(),
                    active: false,
                    handle: Some(workspace),
                });
            }
            Event::Done => {
                state.done = true;
            }
            _ => {}
        }
    }
}

impl Dispatch<ExtWorkspaceGroupHandleV1, ()> for WsState {
    event_created_child!(WsState, ExtWorkspaceGroupHandleV1, []);

    fn event(
        _state: &mut Self,
        _proxy: &ExtWorkspaceGroupHandleV1,
        _event: <ExtWorkspaceGroupHandleV1 as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Groups are intermediate — workspaces come directly from manager
    }
}

impl Dispatch<ExtWorkspaceHandleV1, ()> for WsState {
    fn event(
        state: &mut Self,
        _proxy: &ExtWorkspaceHandleV1,
        event: <ExtWorkspaceHandleV1 as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        use wayland_protocols::ext::workspace::v1::client::ext_workspace_handle_v1::Event;
        let idx = state.workspaces.len().saturating_sub(1);
        match event {
            Event::Name { name } => {
                if let Some(ws) = state.workspaces.get_mut(idx) {
                    ws.name = name;
                }
            }
            Event::State { state: ws_state } => {
                use wayland_protocols::ext::workspace::v1::client::ext_workspace_handle_v1::State;
                let active = ws_state == WEnum::Value(State::Active);
                if let Some(ws) = state.workspaces.get_mut(idx) {
                    ws.active = active;
                }
            }
            Event::Removed =>
            {
                #[allow(clippy::collapsible_match)]
                if idx < state.workspaces.len() {
                    state.workspaces.remove(idx);
                }
            }
            _ => {}
        }
    }
}

fn query_workspaces() -> anyhow::Result<Vec<WorkspaceInfo>> {
    let conn = Connection::connect_to_env()?;
    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();
    let display = conn.display();

    let mut state = WsState {
        manager: None,
        workspaces: Vec::new(),
        done: false,
    };

    let _registry = display.get_registry(&qh, ());

    event_queue.roundtrip(&mut state)?;

    if state.manager.is_none() {
        anyhow::bail!("ext_workspace_manager_v1 not available");
    }
    // Commit and roundtrip — manager reference doesn't live across roundtrips
    {
        let manager = state.manager.as_ref().unwrap();
        manager.commit();
    }
    event_queue.roundtrip(&mut state)?;
    event_queue.roundtrip(&mut state)?;

    Ok(state
        .workspaces
        .iter()
        .enumerate()
        .map(|(i, ws)| WorkspaceInfo {
            id: (i + 1) as u32,
            name: if ws.name.is_empty() {
                format!("workspace-{}", i + 1)
            } else {
                ws.name.clone()
            },
            is_active: ws.active,
        })
        .collect())
}

/// Activate a workspace handle — must be called from within a Wayland event loop.
/// This is a separate function because activate() requires roundtrips for protocol state.
fn activate_workspace(id: u32) -> anyhow::Result<()> {
    let conn = Connection::connect_to_env()?;
    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();
    let display = conn.display();

    let mut state = WsState {
        manager: None,
        workspaces: Vec::new(),
        done: false,
    };

    let _registry = display.get_registry(&qh, ());
    event_queue.roundtrip(&mut state)?;

    if state.manager.is_none() {
        anyhow::bail!("ext_workspace_manager_v1 not available");
    }
    // Commit and roundtrip — manager reference doesn't live across roundtrips
    {
        let manager = state.manager.as_ref().unwrap();
        manager.commit();
    }
    event_queue.roundtrip(&mut state)?;
    event_queue.roundtrip(&mut state)?;

    let idx = id as usize - 1;
    if idx >= state.workspaces.len() {
        anyhow::bail!("workspace {id} not found");
    }

    let handle = state.workspaces[idx]
        .handle
        .take()
        .ok_or_else(|| anyhow::anyhow!("no handle for workspace {id}"))?;

    // Activate the workspace
    handle.activate();
    {
        let manager = state.manager.as_ref().unwrap();
        manager.commit();
    }
    event_queue.roundtrip(&mut state)?;

    Ok(())
}

pub(crate) async fn workspaces_list(_backend: &LabwcBackend) -> anyhow::Result<Vec<WorkspaceInfo>> {
    query_workspaces()
}

pub(crate) async fn workspace_switch(_backend: &LabwcBackend, id: u32) -> anyhow::Result<()> {
    activate_workspace(id)
}

pub(crate) async fn workspace_move_window(
    _backend: &LabwcBackend,
    _window: &str,
    _workspace: u32,
    _follow: bool,
) -> anyhow::Result<()> {
    anyhow::bail!(
        "workspace_move_window not yet implemented — needs ext_workspace_handle_v1.assign()"
    )
}
