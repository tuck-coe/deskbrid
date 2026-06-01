use std::collections::HashMap;

use wayland_client::backend::ObjectId;
use wayland_client::event_created_child;
use wayland_client::protocol::wl_registry::{self, WlRegistry};
use wayland_client::{Connection, Dispatch, Proxy, QueueHandle};

use wayland_protocols::ext::foreign_toplevel_list::v1::client::{
    ext_foreign_toplevel_handle_v1::{self as toplevel_handle, ExtForeignToplevelHandleV1},
    ext_foreign_toplevel_list_v1::{self as toplevel_list, ExtForeignToplevelListV1},
};

use super::types::WindowInfo;

/// State for the Wayland dispatch loop.
pub(crate) struct WlState {
    pub toplevel_list: Option<ExtForeignToplevelListV1>,
    /// Windows indexed by their Wayland ObjectId.
    pub windows: HashMap<ObjectId, WindowInfo>,
    /// Pending window data being accumulated before done event.
    pending: HashMap<ObjectId, PendingWindow>,
    /// Next numeric ID to assign.
    next_id: u64,
    #[allow(dead_code)]
    /// Set to true when finished event is received.
    finished: bool,
}

struct PendingWindow {
    title: Option<String>,
    app_id: Option<String>,
    identifier: Option<String>,
    id: u64,
}

impl Dispatch<WlRegistry, ()> for WlState {
    fn event(
        state: &mut Self,
        registry: &WlRegistry,
        event: <WlRegistry as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
            && interface == "ext_foreign_toplevel_list_v1"
        {
            let list =
                registry.bind::<ExtForeignToplevelListV1, (), Self>(name, version.min(1), qh, ());
            state.toplevel_list = Some(list);
        }
    }
}

impl Dispatch<ExtForeignToplevelListV1, ()> for WlState {
    event_created_child!(WlState, ExtForeignToplevelListV1, [
        0 => (ExtForeignToplevelHandleV1, ())
    ]);

    fn event(
        state: &mut Self,
        _proxy: &ExtForeignToplevelListV1,
        event: <ExtForeignToplevelListV1 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        use toplevel_list::Event;
        match event {
            Event::Toplevel { toplevel } => {
                let obj_id = toplevel.id();
                let window_id = state.next_id;
                state.next_id += 1;
                state.pending.insert(
                    obj_id,
                    PendingWindow {
                        title: None,
                        app_id: None,
                        identifier: None,
                        id: window_id,
                    },
                );
            }
            Event::Finished => {
                state.finished = true;
            }
            _ => {}
        }
    }
}

impl Dispatch<ExtForeignToplevelHandleV1, ()> for WlState {
    fn event(
        state: &mut Self,
        proxy: &ExtForeignToplevelHandleV1,
        event: <ExtForeignToplevelHandleV1 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        use toplevel_handle::Event;
        let obj_id = proxy.id();
        match event {
            Event::Title { title } => {
                if let Some(pending) = state.pending.get_mut(&obj_id) {
                    pending.title = Some(title);
                }
            }
            Event::AppId { app_id } => {
                if let Some(pending) = state.pending.get_mut(&obj_id) {
                    pending.app_id = Some(app_id);
                }
            }
            Event::Identifier { identifier } => {
                if let Some(pending) = state.pending.get_mut(&obj_id) {
                    pending.identifier = Some(identifier);
                }
            }
            Event::Done => {
                if let Some(pending) = state.pending.remove(&obj_id) {
                    // Use stable hash from identifier like cosmic_helper does
                    let ident = pending.identifier.as_deref().unwrap_or("");
                    let numeric_id = if !ident.is_empty() {
                        let mut hash: u64 = 5381;
                        for b in ident.bytes() {
                            hash = hash.wrapping_mul(33).wrapping_add(b as u64);
                        }
                        hash
                    } else {
                        pending.id
                    };
                    let window = WindowInfo {
                        window_id: numeric_id,
                        title: pending.title,
                        app_id: pending.app_id,
                        focused: false,
                        minimized: false,
                        maximized: false,
                        fullscreen: false,
                    };
                    state.windows.insert(obj_id, window);
                }
            }
            Event::Closed => {
                state.windows.remove(&obj_id);
            }
            _ => {}
        }
    }
}

pub(crate) fn list_windows_wayland() -> Vec<WindowInfo> {
    let conn = Connection::connect_to_env().expect("failed to connect to Wayland display");
    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();
    let display = conn.display();

    let mut state = WlState {
        toplevel_list: None,
        windows: HashMap::new(),
        pending: HashMap::new(),
        next_id: 1,
        finished: false,
    };

    let _registry = display.get_registry(&qh, ());

    // Roundtrip to receive global announcements and bind protocol
    event_queue.roundtrip(&mut state).expect("roundtrip failed");

    // Roundtrips to get the toplevel list with all properties
    if state.toplevel_list.is_some() {
        event_queue.roundtrip(&mut state).expect("roundtrip failed");
        event_queue.roundtrip(&mut state).expect("roundtrip failed");
        // Flush remaining events
        event_queue.roundtrip(&mut state).expect("roundtrip failed");
    }

    state.windows.into_values().collect()
}
