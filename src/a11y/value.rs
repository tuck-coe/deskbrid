//! AT-SPI2 Value + EditableText get/set.

use anyhow::Context;
use serde_json::json;
use zbus::zvariant::ObjectPath;

use super::bus::{self, DEST};

pub async fn set_element_value(object_ref: &str, value: &str) -> anyhow::Result<serde_json::Value> {
    let conn = bus::connect_a11y().await?;
    let obj_path: ObjectPath =
        ObjectPath::try_from(object_ref).context("invalid AT-SPI object path")?;

    // Try numeric value first
    if let Ok(num) = value.parse::<f64>() {
        let ok: bool = conn
            .call_method(
                Some(DEST),
                &obj_path,
                Some("org.a11y.atspi.Value"),
                "SetCurrentValue",
                &(num,),
            )
            .await
            .ok()
            .and_then(|r| r.body().deserialize().ok())
            .unwrap_or(false);
        if ok {
            return Ok(json!({"ok": true, "method": "numeric"}));
        }
    }

    // Fall back to EditableText
    let ok: bool = conn
        .call_method(
            Some(DEST),
            &obj_path,
            Some("org.a11y.atspi.EditableText"),
            "SetTextContents",
            &(value,),
        )
        .await
        .ok()
        .and_then(|r| r.body().deserialize().ok())
        .unwrap_or(false);

    if ok {
        Ok(json!({"ok": true, "method": "editable_text"}))
    } else {
        anyhow::bail!(
            "set_value failed: neither Value nor EditableText interface accepted the input"
        )
    }
}

pub async fn get_element_text(
    object_ref: &str,
    max_chars: Option<i32>,
) -> anyhow::Result<serde_json::Value> {
    let max_chars = max_chars.unwrap_or(5000);
    let conn = bus::connect_a11y().await?;
    let obj_path: ObjectPath = ObjectPath::try_from(object_ref)?;

    let char_count: i32 = conn
        .call_method(
            Some(DEST),
            &obj_path,
            Some("org.a11y.atspi.Text"),
            "GetCharacterCount",
            &(),
        )
        .await
        .ok()
        .and_then(|r| r.body().deserialize().ok())
        .unwrap_or(0);

    let content: String = if char_count > 0 {
        conn.call_method(
            Some(DEST),
            &obj_path,
            Some("org.a11y.atspi.Text"),
            "GetText",
            &(0i32, char_count.min(max_chars)),
        )
        .await
        .ok()
        .and_then(|r| r.body().deserialize().ok())
        .unwrap_or_default()
    } else {
        String::new()
    };

    let caret_offset: i32 = conn
        .call_method(
            Some(DEST),
            &obj_path,
            Some("org.a11y.atspi.Text"),
            "GetCaretOffset",
            &(),
        )
        .await
        .ok()
        .and_then(|r| r.body().deserialize().ok())
        .unwrap_or(0);

    Ok(json!({
        "character_count": char_count,
        "caret_offset": caret_offset,
        "content": content,
        "selections": []
    }))
}
