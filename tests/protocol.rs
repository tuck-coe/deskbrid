use deskbrid::protocol::{ClientMessage, ServerMessage, Session};

#[test]
fn action_message_defaults_id() {
    let message: ClientMessage = serde_json::from_str(
        r#"{"type":"action","action":"clipboard:read","params":{}}"#,
    )
    .expect("client message should deserialize");

    match message {
        ClientMessage::Action { id, action, .. } => {
            assert!(!id.is_empty());
            assert_eq!(action, "clipboard:read");
        }
        _ => panic!("expected action message"),
    }
}

#[test]
fn server_event_serializes_as_protocol_shape() {
    let message = ServerMessage::Event {
        event: "clipboard".to_string(),
        data: serde_json::json!({"text":"hello"}),
    };

    let value = serde_json::to_value(message).expect("server message should serialize");
    assert_eq!(value["type"], "event");
    assert_eq!(value["event"], "clipboard");
    assert_eq!(value["data"]["text"], "hello");
}

#[test]
fn session_tracks_subscriptions() {
    let mut session = Session::new();
    session.subscribe("clipboard");
    assert!(session.is_subscribed("clipboard"));
    session.unsubscribe("clipboard");
    assert!(!session.is_subscribed("clipboard"));
}
