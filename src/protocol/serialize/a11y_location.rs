use super::Action;
use serde_json::json;

pub(super) fn serialize_a11y_location(action: &Action, id: &str) -> serde_json::Value {
    match action {
        // Location
        Action::LocationGet => json!({"type": "location.get", "id": id}),
        Action::UiTreeGet => json!({"type":"ui.tree.get","id":id}),
        Action::UiElementClick { selector } => {
            json!({"type":"ui.element.click","id":id,"selector":selector})
        }
        Action::UiElementSetText { selector, text } => {
            json!({"type":"ui.element.set_text","id":id,"selector":selector,"text":text})
        }
        _ => unreachable!("not a a11y_location action"),
    }
}
