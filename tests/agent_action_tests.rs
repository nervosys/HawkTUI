//! `execute_action` must change the program, not merely answer that it will.
//!
//! This exists because it did not. `Command::AgentAction` was constructed by
//! the headless driver and the RPC transport and then matched by an empty arm
//! in all three consumers, under a comment saying the model "should handle
//! these in its update function" — a route that did not exist, because `Model`
//! had no hook to deliver an action to.
//!
//! The defect survived because the only test asserted that the *session*
//! replied `success: true, status: "dispatched"`. That response is truthful
//! about what the session did and says nothing about what the program did. A
//! check is worth what it would fail on, so these tests read the model.

use hawktui::agent::driver::HeadlessDriver;
use hawktui::agent::protocol::{AgentRequest, InjectedEvent};
use hawktui::ontology::registry::{UiNode, UiTree};
use hawktui::prelude::*;

/// `execute_action` addresses a widget in the UI tree, so a model that expects
/// to be driven has to publish one.
fn register_counter_node(registry: &mut OntologyRegistry) {
    registry.set_tree(UiTree::new(
        UiNode::new("Container", SemanticRole::Container).with_id("counter"),
    ));
}

#[derive(Default)]
struct Counter {
    count: i32,
}

enum Msg {
    Add(i32),
}

impl Model for Counter {
    type Msg = Msg;

    fn update(&mut self, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Add(n) => self.count += n,
        }
        Command::None
    }

    fn view(&self, _frame: &mut hawktui::terminal::Frame<'_>) {}

    fn handle_event(&self, _event: Event) -> Option<Msg> {
        None
    }

    fn register_ontology(&self, registry: &mut OntologyRegistry) {
        register_counter_node(registry);
    }

    fn handle_action(&self, action: &str, params: &serde_json::Value) -> Option<Msg> {
        match action {
            "add" => Some(Msg::Add(params["n"].as_i64().unwrap_or(0) as i32)),
            _ => None,
        }
    }
}

/// A model that implements `handle_action` is driven by `execute_action`.
#[test]
fn execute_action_changes_the_model() {
    let mut driver = HeadlessDriver::new(Counter::default(), 20, 5).expect("driver");
    let resp = driver.process_request(&AgentRequest::ExecuteAction {
        agent_id: "counter".into(),
        action: "add".into(),
        params: serde_json::json!({ "n": 7 }),
    });

    assert_eq!(
        driver.model().count,
        7,
        "execute_action returned {resp:?} but the model did not change"
    );
}

/// A model that does not handle the action is told so, rather than told "ok".
#[test]
fn unhandled_action_is_reported_as_unhandled() {
    let mut driver = HeadlessDriver::new(Counter::default(), 20, 5).expect("driver");
    let resp = driver.process_request(&AgentRequest::ExecuteAction {
        agent_id: "counter".into(),
        action: "subtract".into(),
        params: serde_json::json!({ "n": 1 }),
    });

    assert_eq!(
        driver.model().count,
        0,
        "an unhandled action changed the model"
    );
    assert!(
        !resp.success,
        "an action no model handles was reported as successful: {resp:?}"
    );
}

/// The falsifier for both tests above: with no `handle_action`, the first test
/// must fail. This asserts the default hook really is inert, so a future
/// refactor cannot make these pass vacuously.
#[test]
fn the_default_hook_handles_nothing() {
    struct Inert;
    impl Model for Inert {
        type Msg = ();
        fn update(&mut self, _msg: ()) -> Command<()> {
            Command::None
        }
        fn view(&self, _frame: &mut hawktui::terminal::Frame<'_>) {}
        fn handle_event(&self, _event: Event) -> Option<()> {
            None
        }
    }
    assert!(
        Inert
            .handle_action("anything", &serde_json::json!({}))
            .is_none(),
        "the default handle_action claimed to handle an action"
    );
}

/// Injected events already worked; this pins that they still do, so the fix
/// cannot regress the path that was carrying the whole agent story.
#[test]
fn inject_event_still_reaches_the_model() {
    struct Keys {
        seen: u32,
    }
    impl Model for Keys {
        type Msg = ();
        fn update(&mut self, _msg: ()) -> Command<()> {
            self.seen += 1;
            Command::None
        }
        fn view(&self, _frame: &mut hawktui::terminal::Frame<'_>) {}
        fn handle_event(&self, _event: Event) -> Option<()> {
            Some(())
        }
    }
    let mut driver = HeadlessDriver::new(Keys { seen: 0 }, 20, 5).expect("driver");
    driver.process_request(&AgentRequest::InjectEvent {
        event: InjectedEvent::Key {
            code: "a".into(),
            modifiers: vec![],
        },
    });
    assert_eq!(
        driver.model().seen,
        1,
        "injected key did not reach the model"
    );
}
