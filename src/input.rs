//! Input injection — keyboard, mouse, and text input via Mutter.RemoteDesktop.

use anyhow::{anyhow, Context, Result};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;
use zbus::zvariant::OwnedObjectPath;

const MUTTER_DEST: &str = "org.gnome.Mutter.RemoteDesktop";
const MUTTER_PATH: &str = "/org/gnome/Mutter/RemoteDesktop";
const MUTTER_IFACE: &str = "org.gnome.Mutter.RemoteDesktop";
const SESSION_IFACE: &str = "org.gnome.Mutter.RemoteDesktop.Session";
const DEVICE_TYPES_ALL: u32 = 7;
const KEY_RELEASED: u32 = 0;
const KEY_PRESSED: u32 = 1;
const BUTTON_RELEASED: u32 = 0;
const BUTTON_PRESSED: u32 = 1;

#[derive(Clone)]
pub struct InputSession {
    conn: zbus::Connection,
    path: OwnedObjectPath,
    lock: Arc<Mutex<()>>,
}

impl InputSession {
    pub async fn new() -> Result<Self> {
        let conn = zbus::Connection::session()
            .await
            .context("connecting to session bus for input injection")?;
        let proxy = zbus::Proxy::new(&conn, MUTTER_DEST, MUTTER_PATH, MUTTER_IFACE)
            .await
            .context("creating remote desktop proxy")?;

        let path: OwnedObjectPath = proxy
            .call("CreateSession", &())
            .await
            .context("creating remote desktop session")?;

        let session_proxy = zbus::Proxy::new(&conn, MUTTER_DEST, path.as_str(), SESSION_IFACE)
            .await
            .context("creating remote desktop session proxy")?;
        let start_result: Result<(), zbus::Error> =
            session_proxy.call("Start", &(DEVICE_TYPES_ALL)).await;
        if start_result.is_err() {
            let _: () = session_proxy
                .call("Start", &())
                .await
                .context("starting remote desktop session")?;
        }

        Ok(Self {
            conn,
            path,
            lock: Arc::new(Mutex::new(())),
        })
    }

    pub async fn type_text(&self, text: &str) -> Result<()> {
        let _guard = self.lock.lock().await;
        for character in text.chars() {
            match character {
                '\n' => self.tap_key(28).await?,
                '\t' => self.tap_key(15).await?,
                ' ' => self.tap_key(57).await?,
                ch => {
                    let sequence = key_sequence_for_char(ch)
                        .ok_or_else(|| anyhow!("unsupported character for inject:type: {ch:?}"))?;
                    self.send_sequence(&sequence).await?;
                }
            }
        }
        Ok(())
    }

    pub async fn send_keys(&self, keys: &[String]) -> Result<()> {
        if keys.is_empty() {
            return Err(anyhow!("inject:key requires at least one key"));
        }

        let _guard = self.lock.lock().await;
        let mut keycodes = Vec::with_capacity(keys.len());
        for key in keys {
            keycodes.push(keycode_for_name(key).ok_or_else(|| anyhow!("unknown key: {key}"))?);
        }

        for keycode in &keycodes[..keycodes.len().saturating_sub(1)] {
            self.notify_keyboard(KEY_PRESSED, *keycode).await?;
        }
        if let Some(last) = keycodes.last().copied() {
            self.notify_keyboard(KEY_PRESSED, last).await?;
            self.notify_keyboard(KEY_RELEASED, last).await?;
        }
        for keycode in keycodes[..keycodes.len().saturating_sub(1)].iter().rev() {
            self.notify_keyboard(KEY_RELEASED, *keycode).await?;
        }
        Ok(())
    }

    pub async fn mouse_action(&self, params: &Value) -> Result<()> {
        let _guard = self.lock.lock().await;
        let kind = params
            .get("type")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("missing 'type' param"))?;
        match kind {
            "move" => {
                let x = params.get("x").and_then(Value::as_f64).unwrap_or(0.0);
                let y = params.get("y").and_then(Value::as_f64).unwrap_or(0.0);
                self.notify_pointer_motion_absolute(x, y, 1.0, 1.0).await?;
            }
            "click" => {
                let x = params.get("x").and_then(Value::as_f64).unwrap_or(0.0);
                let y = params.get("y").and_then(Value::as_f64).unwrap_or(0.0);
                let button_name = params
                    .get("button")
                    .and_then(Value::as_str)
                    .unwrap_or("left");
                let button = pointer_button_code(button_name)?;
                self.notify_pointer_motion_absolute(x, y, 1.0, 1.0).await?;
                self.notify_pointer_button(button, BUTTON_PRESSED).await?;
                self.notify_pointer_button(button, BUTTON_RELEASED).await?;
            }
            "scroll" => {
                let dx = params.get("dx").and_then(Value::as_f64).unwrap_or(0.0);
                let dy = params.get("dy").and_then(Value::as_f64).unwrap_or(0.0);
                if dx != 0.0 {
                    self.notify_pointer_axis(0, dx, true).await?;
                }
                if dy != 0.0 {
                    self.notify_pointer_axis(1, dy, true).await?;
                }
            }
            other => return Err(anyhow!("unsupported mouse action: {other}")),
        }
        Ok(())
    }

    async fn send_sequence(&self, sequence: &[(u32, bool)]) -> Result<()> {
        for (keycode, shift) in sequence {
            if *shift {
                self.notify_keyboard(KEY_PRESSED, 42).await?;
            }
            self.tap_key(*keycode).await?;
            if *shift {
                self.notify_keyboard(KEY_RELEASED, 42).await?;
            }
        }
        Ok(())
    }

    async fn tap_key(&self, keycode: u32) -> Result<()> {
        self.notify_keyboard(KEY_PRESSED, keycode).await?;
        self.notify_keyboard(KEY_RELEASED, keycode).await?;
        Ok(())
    }

    async fn notify_keyboard(&self, keystate: u32, keycode: u32) -> Result<()> {
        let proxy = self.session_proxy().await?;
        let _: () = proxy
            .call("NotifyKeyboard", &(keystate, keycode))
            .await
            .with_context(|| format!("injecting keyboard event keystate={keystate} keycode={keycode}"))?;
        Ok(())
    }

    async fn notify_pointer_motion_absolute(
        &self,
        x: f64,
        y: f64,
        stream_width: f64,
        stream_height: f64,
    ) -> Result<()> {
        let proxy = self.session_proxy().await?;
        let _: () = proxy
            .call(
                "NotifyPointerMotionAbsolute",
                &(x, y, stream_width, stream_height),
            )
            .await
            .context("injecting absolute pointer motion")?;
        Ok(())
    }

    async fn notify_pointer_button(&self, button: u32, state: u32) -> Result<()> {
        let proxy = self.session_proxy().await?;
        let _: () = proxy
            .call("NotifyPointerButton", &(button, state))
            .await
            .context("injecting pointer button event")?;
        Ok(())
    }

    async fn notify_pointer_axis(&self, axis: u32, value: f64, finish: bool) -> Result<()> {
        let proxy = self.session_proxy().await?;
        let _: () = proxy
            .call("NotifyPointerAxis", &(axis, value, finish))
            .await
            .context("injecting pointer axis event")?;
        Ok(())
    }

    async fn session_proxy(&self) -> Result<zbus::Proxy<'_>> {
        debug!("using input session {}", self.path.as_str());
        zbus::Proxy::new(&self.conn, MUTTER_DEST, self.path.as_str(), SESSION_IFACE)
            .await
            .context("creating session proxy")
    }
}

fn pointer_button_code(button: &str) -> Result<u32> {
    match button {
        "left" => Ok(0x110),
        "right" => Ok(0x111),
        "middle" => Ok(0x112),
        other => Err(anyhow!("unsupported pointer button: {other}")),
    }
}

fn key_sequence_for_char(character: char) -> Option<Vec<(u32, bool)>> {
    let lowercase = character.to_ascii_lowercase();
    let keycode = match lowercase {
        'a' => 30,
        'b' => 48,
        'c' => 46,
        'd' => 32,
        'e' => 18,
        'f' => 33,
        'g' => 34,
        'h' => 35,
        'i' => 23,
        'j' => 36,
        'k' => 37,
        'l' => 38,
        'm' => 50,
        'n' => 49,
        'o' => 24,
        'p' => 25,
        'q' => 16,
        'r' => 19,
        's' => 31,
        't' => 20,
        'u' => 22,
        'v' => 47,
        'w' => 17,
        'x' => 45,
        'y' => 21,
        'z' => 44,
        '0' => 11,
        '1' => 2,
        '2' => 3,
        '3' => 4,
        '4' => 5,
        '5' => 6,
        '6' => 7,
        '7' => 8,
        '8' => 9,
        '9' => 10,
        _ => return None,
    };
    Some(vec![(keycode, character.is_ascii_uppercase())])
}

fn keycode_for_name(name: &str) -> Option<u32> {
    match name.to_ascii_lowercase().as_str() {
        "a" => Some(30),
        "b" => Some(48),
        "c" => Some(46),
        "d" => Some(32),
        "e" => Some(18),
        "f" => Some(33),
        "g" => Some(34),
        "h" => Some(35),
        "i" => Some(23),
        "j" => Some(36),
        "k" => Some(37),
        "l" => Some(38),
        "m" => Some(50),
        "n" => Some(49),
        "o" => Some(24),
        "p" => Some(25),
        "q" => Some(16),
        "r" => Some(19),
        "s" => Some(31),
        "t" => Some(20),
        "u" => Some(22),
        "v" => Some(47),
        "w" => Some(17),
        "x" => Some(45),
        "y" => Some(21),
        "z" => Some(44),
        "0" => Some(11),
        "1" => Some(2),
        "2" => Some(3),
        "3" => Some(4),
        "4" => Some(5),
        "5" => Some(6),
        "6" => Some(7),
        "7" => Some(8),
        "8" => Some(9),
        "9" => Some(10),
        "enter" => Some(28),
        "tab" => Some(15),
        "escape" => Some(1),
        "backspace" => Some(14),
        "space" => Some(57),
        "ctrl" => Some(29),
        "shift" => Some(42),
        "alt" => Some(56),
        "super" => Some(125),
        "left" => Some(105),
        "right" => Some(106),
        "up" => Some(103),
        "down" => Some(108),
        "f1" => Some(59),
        "f2" => Some(60),
        "f3" => Some(61),
        "f4" => Some(62),
        "f5" => Some(63),
        "f6" => Some(64),
        "f7" => Some(65),
        "f8" => Some(66),
        "f9" => Some(67),
        "f10" => Some(68),
        "f11" => Some(87),
        "f12" => Some(88),
        _ => None,
    }
}
