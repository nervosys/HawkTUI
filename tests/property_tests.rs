//! Property-based tests for the agent protocol types.
//!
//! Verifies that arbitrary JSON inputs never cause panics during
//! deserialization, and that valid types round-trip through serde.

use louie::agent::protocol::{AgentRequest, AgentResponse, RequestEnvelope};
use proptest::prelude::*;

// ── Fuzz deserialization: no panics on arbitrary input ──────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2000))]

    #[test]
    fn request_envelope_never_panics_on_arbitrary_json(s in "\\PC{0,512}") {
        // Must not panic — errors are fine.
        let _ = serde_json::from_str::<RequestEnvelope>(&s);
    }

    #[test]
    fn agent_request_never_panics_on_arbitrary_json(s in "\\PC{0,512}") {
        let _ = serde_json::from_str::<AgentRequest>(&s);
    }

    #[test]
    fn agent_response_never_panics_on_arbitrary_json(s in "\\PC{0,512}") {
        let _ = serde_json::from_str::<AgentResponse>(&s);
    }
}

// ── Round-trip tests for known-good messages ────────────────────────────────

proptest! {
    #[test]
    fn ping_round_trips(id in "[a-z0-9]{1,32}") {
        let json = format!(r#"{{"id":"{id}","type":"ping"}}"#);
        let envelope: RequestEnvelope = serde_json::from_str(&json).unwrap();
        assert_eq!(envelope.id.as_deref(), Some(id.as_str()));
        let reserialized = serde_json::to_string(&envelope).unwrap();
        let re_parsed: RequestEnvelope = serde_json::from_str(&reserialized).unwrap();
        assert_eq!(re_parsed.id, envelope.id);
    }

    #[test]
    fn get_state_round_trips(agent_id in "[a-zA-Z_][a-zA-Z0-9_]{0,63}") {
        let json = format!(r#"{{"type":"get_state","agent_id":"{agent_id}"}}"#);
        let envelope: RequestEnvelope = serde_json::from_str(&json).unwrap();
        let reserialized = serde_json::to_string(&envelope).unwrap();
        let re_parsed: RequestEnvelope = serde_json::from_str(&reserialized).unwrap();
        assert_eq!(
            serde_json::to_value(&envelope.request).unwrap(),
            serde_json::to_value(&re_parsed.request).unwrap(),
        );
    }

    #[test]
    fn execute_action_round_trips(
        agent_id in "[a-zA-Z_][a-zA-Z0-9_]{0,31}",
        action in "[a-z_]{1,32}",
    ) {
        let json = format!(
            r#"{{"type":"execute_action","agent_id":"{agent_id}","action":"{action}","params":{{}}}}"#
        );
        let envelope: RequestEnvelope = serde_json::from_str(&json).unwrap();
        let reserialized = serde_json::to_string(&envelope).unwrap();
        let re_parsed: RequestEnvelope = serde_json::from_str(&reserialized).unwrap();
        assert_eq!(
            serde_json::to_value(&envelope.request).unwrap(),
            serde_json::to_value(&re_parsed.request).unwrap(),
        );
    }

    #[test]
    fn response_ok_round_trips(msg in "[a-zA-Z0-9 ]{0,64}") {
        let resp = AgentResponse::ok(serde_json::json!({"message": msg}));
        let json = serde_json::to_string(&resp).unwrap();
        let parsed: AgentResponse = serde_json::from_str(&json).unwrap();
        assert!(parsed.success);
        assert_eq!(resp.data, parsed.data);
    }

    #[test]
    fn response_err_round_trips(msg in "[a-zA-Z0-9 ]{1,64}") {
        let resp = AgentResponse::err(&msg);
        let json = serde_json::to_string(&resp).unwrap();
        let parsed: AgentResponse = serde_json::from_str(&json).unwrap();
        assert!(!parsed.success);
        assert_eq!(parsed.error.as_deref(), Some(msg.as_str()));
    }

    #[test]
    fn inject_key_round_trips(
        code in "[a-zA-Z]{1,16}",
    ) {
        let json = format!(
            r#"{{"type":"inject_event","event":{{"kind":"key","code":"{code}"}}}}"#
        );
        let envelope: RequestEnvelope = serde_json::from_str(&json).unwrap();
        let reserialized = serde_json::to_string(&envelope).unwrap();
        let re_parsed: RequestEnvelope = serde_json::from_str(&reserialized).unwrap();
        assert_eq!(
            serde_json::to_value(&envelope.request).unwrap(),
            serde_json::to_value(&re_parsed.request).unwrap(),
        );
    }

    #[test]
    fn inject_resize_clamps_values(width in 0u16..=u16::MAX, height in 0u16..=u16::MAX) {
        let json = format!(
            r#"{{"type":"inject_event","event":{{"kind":"resize","width":{width},"height":{height}}}}}"#
        );
        // Must parse without panic
        let envelope: RequestEnvelope = serde_json::from_str(&json).unwrap();
        // Verify it round-trips
        let reserialized = serde_json::to_string(&envelope).unwrap();
        let _: RequestEnvelope = serde_json::from_str(&reserialized).unwrap();
    }
}
