//! AT-SPI2 accessibility tree snapshot builder.
//! BFS traversal with full node data: bounds, actions, value, text.

use serde::Serialize;
use serde_json::json;
use std::collections::VecDeque;
use zbus::zvariant::ObjectPath;

use super::bus::{self, ROOT, child_path, element_json};

#[derive(Debug, Clone, Serialize)]
pub struct AccessibilityNode {
    pub index: u32,
    pub parent_index: Option<u32>,
    pub depth: u32,
    pub object_ref: String,
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub child_count: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounds: Option<Bounds>,
    pub states: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<AccessibilityAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<AccessibilityValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<AccessibilityText>,
    pub supports_editable_text: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Bounds {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct AccessibilityAction {
    pub index: i32,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AccessibilityValue {
    pub current: f64,
    pub minimum: f64,
    pub maximum: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AccessibilityText {
    pub character_count: i32,
    pub caret_offset: i32,
    pub content: String,
    pub selections: Vec<i32>,
}

/// Build a full accessibility tree snapshot matching computer-use-linux output shape.
pub async fn snapshot_tree(
    app_name: Option<&str>,
    _pid: Option<u32>,
    max_nodes: Option<usize>,
    max_depth: Option<u32>,
) -> anyhow::Result<serde_json::Value> {
    let max_nodes = max_nodes.unwrap_or(200);
    let max_depth = max_depth.unwrap_or(10) as usize;
    let conn = bus::connect_a11y().await?;
    let root: ObjectPath = ObjectPath::try_from(ROOT)?;

    let mut nodes: Vec<AccessibilityNode> = Vec::new();
    let mut queue: VecDeque<(ObjectPath<'static>, usize, Option<u32>)> = VecDeque::new();
    queue.push_back((root.into_owned(), 0, None));

    while let Some((path, depth, parent_idx)) = queue.pop_front() {
        if nodes.len() >= max_nodes || depth > max_depth {
            continue;
        }

        let info = element_json(&conn, &path).await;
        let role_str = info["role"].as_str().unwrap_or("unknown");
        let name_str = info["name"].as_str().map(|s| s.to_string());

        if let Some(filter) = app_name
            && let Some(ref node_name) = name_str
            && !node_name.to_lowercase().contains(&filter.to_lowercase())
        {
            continue;
        }

        let bounds = get_bounds(&conn, &path).await;
        let actions = get_actions(&conn, &path).await;
        let value = get_value_proxy(&conn, &path).await;
        let text = get_text_data(&conn, &path, 500).await;
        let has_editable = check_editable_text(&conn, &path).await;

        let child_count = info["child_count"].as_i64().unwrap_or(0) as i32;
        let states = info["states"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let node_index = nodes.len() as u32;
        let node = AccessibilityNode {
            index: node_index,
            parent_index: parent_idx,
            depth: depth as u32,
            object_ref: path.to_string(),
            role: role_str.to_string(),
            name: name_str,
            description: info["description"].as_str().map(|s| s.to_string()),
            child_count,
            bounds,
            states,
            actions,
            value,
            text,
            supports_editable_text: has_editable,
        };

        nodes.push(node);

        let cc = child_count.min(50);
        for i in 0..cc {
            if let Some(cp) = child_path(&conn, &path, i).await {
                queue.push_back((cp, depth + 1, Some(node_index)));
            }
        }
    }

    Ok(json!({"nodes": nodes, "count": nodes.len()}))
}

pub(crate) async fn get_bounds(conn: &zbus::Connection, path: &ObjectPath<'_>) -> Option<Bounds> {
    // Component/GetExtents returns (x, y, width, height) as a struct
    let reply = conn
        .call_method(
            Some(bus::DEST),
            path,
            Some("org.a11y.atspi.Component"),
            "GetExtents",
            &(0u32),
        )
        .await
        .ok()?;

    let body = reply.body();
    let (x, y, width, height): (i32, i32, i32, i32) = body.deserialize().ok()?;

    Some(Bounds {
        x,
        y,
        width,
        height,
    })
}

async fn get_actions(conn: &zbus::Connection, path: &ObjectPath<'_>) -> Vec<AccessibilityAction> {
    let action_count: i32 = conn
        .call_method(
            Some(bus::DEST),
            path,
            Some("org.a11y.atspi.Action"),
            "GetActionCount",
            &(),
        )
        .await
        .ok()
        .and_then(|r| r.body().deserialize().ok())
        .unwrap_or(0);

    let mut actions = Vec::with_capacity(action_count as usize);
    for i in 0..action_count {
        let name: String = conn
            .call_method(
                Some(bus::DEST),
                path,
                Some("org.a11y.atspi.Action"),
                "GetName",
                &(i,),
            )
            .await
            .ok()
            .and_then(|r| r.body().deserialize().ok())
            .unwrap_or_default();
        let description: String = conn
            .call_method(
                Some(bus::DEST),
                path,
                Some("org.a11y.atspi.Action"),
                "GetDescription",
                &(i,),
            )
            .await
            .ok()
            .and_then(|r| r.body().deserialize().ok())
            .unwrap_or_default();
        actions.push(AccessibilityAction {
            index: i,
            name,
            description,
        });
    }
    actions
}

async fn get_value_proxy(
    conn: &zbus::Connection,
    path: &ObjectPath<'_>,
) -> Option<AccessibilityValue> {
    let current: f64 = conn
        .call_method(
            Some(bus::DEST),
            path,
            Some("org.a11y.atspi.Value"),
            "GetCurrentValue",
            &(),
        )
        .await
        .ok()
        .and_then(|r| r.body().deserialize().ok())?;

    let minimum: f64 = conn
        .call_method(
            Some(bus::DEST),
            path,
            Some("org.a11y.atspi.Value"),
            "GetMinimumValue",
            &(),
        )
        .await
        .ok()
        .and_then(|r| r.body().deserialize().ok())
        .unwrap_or(0.0);

    let maximum: f64 = conn
        .call_method(
            Some(bus::DEST),
            path,
            Some("org.a11y.atspi.Value"),
            "GetMaximumValue",
            &(),
        )
        .await
        .ok()
        .and_then(|r| r.body().deserialize().ok())
        .unwrap_or(0.0);

    Some(AccessibilityValue {
        current,
        minimum,
        maximum,
    })
}

async fn get_text_data(
    conn: &zbus::Connection,
    path: &ObjectPath<'_>,
    max_chars: i32,
) -> Option<AccessibilityText> {
    let char_count: i32 = conn
        .call_method(
            Some(bus::DEST),
            path,
            Some("org.a11y.atspi.Text"),
            "GetCharacterCount",
            &(),
        )
        .await
        .ok()
        .and_then(|r| r.body().deserialize().ok())?;

    if char_count == 0 {
        return Some(AccessibilityText {
            character_count: 0,
            caret_offset: 0,
            content: String::new(),
            selections: Vec::new(),
        });
    }

    let content: String = conn
        .call_method(
            Some(bus::DEST),
            path,
            Some("org.a11y.atspi.Text"),
            "GetText",
            &(0i32, char_count.min(max_chars)),
        )
        .await
        .ok()
        .and_then(|r| r.body().deserialize().ok())
        .unwrap_or_default();

    let caret: i32 = conn
        .call_method(
            Some(bus::DEST),
            path,
            Some("org.a11y.atspi.Text"),
            "GetCaretOffset",
            &(),
        )
        .await
        .ok()
        .and_then(|r| r.body().deserialize().ok())
        .unwrap_or(0);

    Some(AccessibilityText {
        character_count: char_count,
        caret_offset: caret,
        content,
        selections: Vec::new(),
    })
}

async fn check_editable_text(conn: &zbus::Connection, path: &ObjectPath<'_>) -> bool {
    conn.call_method(
        Some(bus::DEST),
        path,
        Some("org.a11y.atspi.EditableText"),
        "SetTextContents",
        &("test"),
    )
    .await
    .is_ok()
}
