// deskbrid@deskbrid — GNOME Shell extension for window management over DBus
// Provides: ListWindows, FocusedWindow, FocusWindow, WindowStateChanged signal

const { Gio, GLib, Meta, Shell } = imports.gi;
const Main = imports.ui.main;

const DBUS_SERVICE = 'org.deskbrid.WindowManager';
const DBUS_PATH = '/org/deskbrid/WindowManager';
const DBUS_IFACE = 'org.deskbrid.WindowManager';

const DeskbridIface = `
<node>
  <interface name="${DBUS_IFACE}">
    <method name="ListWindows">
      <arg type="s" name="result" direction="out"/>
    </method>
    <method name="FocusedWindow">
      <arg type="s" name="result" direction="out"/>
    </method>
    <method name="FocusWindow">
      <arg type="s" name="app_id" direction="in"/>
      <arg type="s" name="title" direction="in"/>
      <arg type="b" name="exact" direction="in"/>
      <arg type="b" name="success" direction="out"/>
    </method>
    <signal name="WindowStateChanged">
      <arg type="s" name="window_info"/>
    </signal>
  </interface>
</node>`;

function serializeWindows(windows) {
    return JSON.stringify(windows.map(w => {
        const m = w.meta_window;
        const rect = m.get_frame_rect();
        return {
            title: m.get_title() || '',
            app_id: m.get_wm_class() || '',
            pid: m.get_pid() || 0,
            workspace: m.get_workspace() ? m.get_workspace().index() : 0,
            focused: global.display.focus_window === m,
            geometry: [rect.x, rect.y, rect.width, rect.height],
            wm_class: m.get_wm_class() || ''
        };
    }));
}

function serializeFocusedWindow() {
    const m = global.display.focus_window;
    if (!m) return 'null';
    const rect = m.get_frame_rect();
    return JSON.stringify({
        title: m.get_title() || '',
        app_id: m.get_wm_class() || '',
        pid: m.get_pid() || 0,
        workspace: m.get_workspace() ? m.get_workspace().index() : 0,
        focused: true,
        geometry: [rect.x, rect.y, rect.width, rect.height],
        wm_class: m.get_wm_class() || ''
    });
}

let _debounceTimer = 0;
let _dbusImpl = null;
let _dbusId = null;

function emitWindowStateChanged() {
    if (_debounceTimer) {
        GLib.source_remove(_debounceTimer);
        _debounceTimer = 0;
    }
    _debounceTimer = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 150, () => {
        _debounceTimer = 0;
        try {
            const info = serializeFocusedWindow();
            _dbusImpl.emit_signal('WindowStateChanged',
                GLib.Variant.new('(s)', [info]));
        } catch (e) {
            log('[deskbrid] signal error: ' + e);
        }
        return false; // one-shot
    });
}

function init() {
    // Hook up window focus tracking
}

function enable() {
    // Register DBus service
    _dbusImpl = Gio.DBusExportedObject.wrapJSObject(DeskbridIface, {
        ListWindows() {
            return serializeWindows(global.get_window_actors());
        },
        FocusedWindow() {
            return serializeFocusedWindow();
        },
        FocusWindow(app_id, title, exact) {
            const windows = global.get_window_actors().map(w => w.meta_window);
            const matches = (needle, value) =>
                exact ? value === needle : value.toLowerCase().includes(needle.toLowerCase());

            const found = windows.find(w => {
                const wmClass = w.get_wm_class() || '';
                const windowTitle = w.get_title() || '';
                if (app_id && matches(app_id, wmClass)) return true;
                if (title && matches(title, windowTitle)) return true;
                return false;
            });

            if (!found) return false;
            found.activate(global.get_current_time());
            return true;
        }
    });

    _dbusId = Gio.DbusUtils.own_name(
        DBUS_SERVICE,
        Gio.BusNameAppOwnershipFlags.NONE,
        null,
        null,
        null,
        null
    );

    _dbusImpl.export(Gio.DBus.session, DBUS_PATH);

    // Watch for window focus changes in GNOME Shell
    const display = global.display;
    this._focusSignalId = display.connect('notify::focus-window', () => {
        emitWindowStateChanged();
    });

    // Also watch window tracking events
    this._windowCreatedId = display.connect('window-created', () => {
        emitWindowStateChanged();
    });

    log('[deskbrid] extension enabled — DBus service: ' + DBUS_SERVICE);
}

function disable() {
    if (this._focusSignalId) {
        global.display.disconnect(this._focusSignalId);
        this._focusSignalId = 0;
    }
    if (this._windowCreatedId) {
        global.display.disconnect(this._windowCreatedId);
        this._windowCreatedId = 0;
    }
    if (_dbusImpl) {
        _dbusImpl.unexport();
        _dbusImpl = null;
    }
    if (_dbusId) {
        Gio.DbusUtils.unown_name(_dbusId);
        _dbusId = null;
    }
    if (_debounceTimer) {
        GLib.source_remove(_debounceTimer);
        _debounceTimer = 0;
    }
    log('[deskbrid] extension disabled');
}
