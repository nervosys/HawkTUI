//! v1.0 test coverage: agent session, headless driver, ontology registry,
//! action validation, focus manager, overlay stack, and error types.

#[cfg(test)]
mod tests {

    // ── Agent Session ────────────────────────────────────────────────

    mod agent_session {
        use louie::agent::protocol::{AgentRequest, InjectedEvent, PROTOCOL_VERSION};
        use louie::agent::session::AgentSession;
        use louie::ontology::registry::{OntologyRegistry, UiNode, UiTree};
        use louie::ontology::SemanticRole;

        fn empty_registry() -> OntologyRegistry {
            OntologyRegistry::new()
        }

        fn registry_with_tree() -> OntologyRegistry {
            let mut reg = OntologyRegistry::new();
            let root = UiNode::new("Container", SemanticRole::Container)
                .with_id("root")
                .with_child(
                    UiNode::new("Input", SemanticRole::Input)
                        .with_id("input-1")
                        .with_label("Name field")
                        .with_state(serde_json::json!({"text": "hello"})),
                );
            reg.set_tree(UiTree::new(root));
            reg
        }

        #[test]
        fn ping_returns_protocol_version() {
            let mut session = AgentSession::new();
            let (resp, quit) = session.process_request(&AgentRequest::Ping, &empty_registry());
            assert!(resp.success);
            assert!(!quit);
            let data = resp.data.unwrap();
            assert_eq!(data["status"], "pong");
            assert_eq!(data["protocol_version"], PROTOCOL_VERSION);
        }

        #[test]
        fn quit_returns_should_quit() {
            let mut session = AgentSession::new();
            let (resp, quit) = session.process_request(&AgentRequest::Quit, &empty_registry());
            assert!(resp.success);
            assert!(quit);
            assert_eq!(resp.data.unwrap()["status"], "quitting");
        }

        #[test]
        fn get_tree_returns_null_when_no_tree() {
            let mut session = AgentSession::new();
            let (resp, _) = session.process_request(&AgentRequest::GetTree, &empty_registry());
            assert!(resp.success);
            assert_eq!(resp.data.unwrap(), serde_json::Value::Null);
        }

        #[test]
        fn get_tree_returns_tree_json() {
            let mut session = AgentSession::new();
            let reg = registry_with_tree();
            let (resp, _) = session.process_request(&AgentRequest::GetTree, &reg);
            assert!(resp.success);
            let data = resp.data.unwrap();
            assert_eq!(data["root"]["widget_type"], "Container");
        }

        #[test]
        fn get_state_found() {
            let mut session = AgentSession::new();
            let reg = registry_with_tree();
            let (resp, _) = session.process_request(
                &AgentRequest::GetState {
                    agent_id: "input-1".into(),
                },
                &reg,
            );
            assert!(resp.success);
            let data = resp.data.unwrap();
            assert_eq!(data["agent_id"], "input-1");
            assert_eq!(data["state"]["text"], "hello");
        }

        #[test]
        fn get_state_not_found() {
            let mut session = AgentSession::new();
            let (resp, _) = session.process_request(
                &AgentRequest::GetState {
                    agent_id: "nonexistent".into(),
                },
                &empty_registry(),
            );
            assert!(!resp.success);
            assert!(resp.error.unwrap().contains("Widget not found"));
        }

        #[test]
        fn get_schema_not_found() {
            let mut session = AgentSession::new();
            let (resp, _) = session.process_request(
                &AgentRequest::GetSchema {
                    widget_type: "NoSuchWidget".into(),
                },
                &empty_registry(),
            );
            assert!(!resp.success);
            assert!(resp.error.unwrap().contains("Unknown widget type"));
        }

        #[test]
        fn query_ontology_empty() {
            let mut session = AgentSession::new();
            let (resp, _) = session.process_request(
                &AgentRequest::QueryOntology {
                    query: None,
                    role: None,
                },
                &empty_registry(),
            );
            assert!(resp.success);
            // Empty registry returns empty array
            let data = resp.data.unwrap();
            assert!(data.as_array().unwrap().is_empty());
        }

        #[test]
        fn subscribe_and_check() {
            let mut session = AgentSession::new();
            let (resp, _) = session.process_request(
                &AgentRequest::Subscribe {
                    events: vec!["state_changed".into(), "render_update".into()],
                },
                &empty_registry(),
            );
            assert!(resp.success);
            assert!(session.is_subscribed("state_changed"));
            assert!(session.is_subscribed("render_update"));
            assert!(!session.is_subscribed("key_press"));
        }

        #[test]
        fn wildcard_subscription() {
            let mut session = AgentSession::new();
            session.process_request(
                &AgentRequest::Subscribe {
                    events: vec!["*".into()],
                },
                &empty_registry(),
            );
            assert!(session.is_subscribed("anything"));
            assert!(session.is_subscribed("state_changed"));
        }

        #[test]
        fn unsubscribe_removes_events() {
            let mut session = AgentSession::new();
            session.process_request(
                &AgentRequest::Subscribe {
                    events: vec!["a".into(), "b".into(), "c".into()],
                },
                &empty_registry(),
            );
            session.process_request(
                &AgentRequest::Unsubscribe {
                    events: vec!["b".into()],
                },
                &empty_registry(),
            );
            assert!(session.is_subscribed("a"));
            assert!(!session.is_subscribed("b"));
            assert!(session.is_subscribed("c"));
        }

        #[test]
        fn subscription_limit_enforced() {
            let mut session = AgentSession::new();
            // Subscribe up to the limit (100)
            let events: Vec<String> = (0..100).map(|i| format!("evt_{i}")).collect();
            let (resp, _) = session.process_request(
                &AgentRequest::Subscribe { events },
                &empty_registry(),
            );
            assert!(resp.success);

            // Attempting to add one more should fail
            let (resp, _) = session.process_request(
                &AgentRequest::Subscribe {
                    events: vec!["overflow".into()],
                },
                &empty_registry(),
            );
            assert!(!resp.success);
            assert!(resp.error.unwrap().contains("limit"));
        }

        #[test]
        fn inject_event_acknowledged() {
            let mut session = AgentSession::new();
            let (resp, _) = session.process_request(
                &AgentRequest::InjectEvent {
                    event: InjectedEvent::Key {
                        code: "enter".into(),
                        modifiers: vec![],
                    },
                },
                &empty_registry(),
            );
            assert!(resp.success);
            assert_eq!(resp.data.unwrap()["status"], "injected");
        }

        #[test]
        fn execute_action_widget_not_found() {
            let mut session = AgentSession::new();
            let (resp, _) = session.process_request(
                &AgentRequest::ExecuteAction {
                    agent_id: "missing".into(),
                    action: "click".into(),
                    params: serde_json::Value::Null,
                },
                &registry_with_tree(),
            );
            assert!(!resp.success);
            assert!(resp.error.unwrap().contains("Widget not found"));
        }

        #[test]
        fn execute_action_dispatched() {
            let mut session = AgentSession::new();
            let reg = registry_with_tree();
            let (resp, _) = session.process_request(
                &AgentRequest::ExecuteAction {
                    agent_id: "input-1".into(),
                    action: "set_text".into(),
                    params: serde_json::json!({"text": "world"}),
                },
                &reg,
            );
            assert!(resp.success);
            assert_eq!(resp.data.unwrap()["status"], "dispatched");
        }

        // ── Event conversion ────────────────────────────────────────

        #[test]
        fn convert_key_event() {
            let ev = InjectedEvent::Key {
                code: "enter".into(),
                modifiers: vec!["ctrl".into()],
            };
            let result = AgentSession::convert_injected_event(&ev);
            assert!(result.is_some());
        }

        #[test]
        fn convert_mouse_click() {
            let ev = InjectedEvent::MouseClick {
                x: 10,
                y: 5,
                button: "left".into(),
            };
            let result = AgentSession::convert_injected_event(&ev);
            assert!(result.is_some());
        }

        #[test]
        fn convert_paste_event() {
            let ev = InjectedEvent::Paste {
                text: "hello".into(),
            };
            let result = AgentSession::convert_injected_event(&ev);
            assert!(result.is_some());
        }

        #[test]
        fn convert_resize_clamps_to_max() {
            use louie::event::Event;
            let ev = InjectedEvent::Resize {
                width: 9999,
                height: 9999,
            };
            let result = AgentSession::convert_injected_event(&ev).unwrap();
            match result {
                Event::Resize(w, h) => {
                    assert_eq!(w, 1024);
                    assert_eq!(h, 1024);
                }
                _ => panic!("expected Resize event"),
            }
        }

        #[test]
        fn convert_resize_clamps_to_min() {
            use louie::event::Event;
            let ev = InjectedEvent::Resize {
                width: 0,
                height: 0,
            };
            let result = AgentSession::convert_injected_event(&ev).unwrap();
            match result {
                Event::Resize(w, h) => {
                    assert_eq!(w, 1);
                    assert_eq!(h, 1);
                }
                _ => panic!("expected Resize event"),
            }
        }

        #[test]
        fn convert_unknown_key_returns_none() {
            let ev = InjectedEvent::Key {
                code: "unknown_key_xyz".into(),
                modifiers: vec![],
            };
            assert!(AgentSession::convert_injected_event(&ev).is_none());
        }

        // ── Emit helpers ────────────────────────────────────────────

        #[test]
        fn emit_state_changed_when_subscribed() {
            let mut session = AgentSession::new();
            session.process_request(
                &AgentRequest::Subscribe {
                    events: vec!["state_changed".into()],
                },
                &empty_registry(),
            );
            let ev = session.emit_state_changed("w1", serde_json::json!({"v": 1}));
            assert!(ev.is_some());
        }

        #[test]
        fn emit_state_changed_when_not_subscribed() {
            let session = AgentSession::new();
            let ev = session.emit_state_changed("w1", serde_json::json!({"v": 1}));
            assert!(ev.is_none());
        }

        #[test]
        fn emit_render_update_with_wildcard() {
            let mut session = AgentSession::new();
            session.process_request(
                &AgentRequest::Subscribe {
                    events: vec!["*".into()],
                },
                &empty_registry(),
            );
            let ev = session.emit_render_update(serde_json::json!({}));
            assert!(ev.is_some());
        }

        #[test]
        fn emit_render_update_when_not_subscribed() {
            let session = AgentSession::new();
            let ev = session.emit_render_update(serde_json::json!({}));
            assert!(ev.is_none());
        }
    }

    // ── Headless Driver ──────────────────────────────────────────────

    mod headless_driver {
        use louie::agent::driver::HeadlessDriver;
        use louie::agent::protocol::{AgentRequest, RequestEnvelope};
        use louie::event::Event;
        use louie::runtime::{Command, Model};
        use louie::terminal::Frame;

        /// Minimal Model for driver tests.
        struct TestApp {
            counter: i32,
        }

        #[derive(Debug)]
        enum TestMsg {
            Increment,
            Quit,
        }

        impl Model for TestApp {
            type Msg = TestMsg;

            fn update(&mut self, msg: TestMsg) -> Command<TestMsg> {
                match msg {
                    TestMsg::Increment => {
                        self.counter += 1;
                        Command::None
                    }
                    TestMsg::Quit => Command::Quit,
                }
            }

            fn view(&self, frame: &mut Frame<'_>) {
                let area = frame.area();
                let text = format!("Counter: {}", self.counter);
                frame.render_widget(text.as_str(), area);
            }

            fn handle_event(&self, event: Event) -> Option<TestMsg> {
                match event {
                    Event::Key(ke) => match ke.code {
                        louie::event::KeyCode::Char('q') => Some(TestMsg::Quit),
                        louie::event::KeyCode::Up => Some(TestMsg::Increment),
                        _ => None,
                    },
                    Event::Tick => Some(TestMsg::Increment),
                    _ => None,
                }
            }

            fn init(&self) -> Command<TestMsg> {
                Command::None
            }
        }

        #[test]
        fn driver_creation_and_running() {
            let app = TestApp { counter: 0 };
            let driver = HeadlessDriver::new(app, 80, 24).unwrap();
            assert!(driver.is_running());
            assert_eq!(driver.model().counter, 0);
        }

        #[test]
        fn driver_ping() {
            let app = TestApp { counter: 0 };
            let mut driver = HeadlessDriver::new(app, 80, 24).unwrap();
            let resp = driver.process_request(&AgentRequest::Ping);
            assert!(resp.success);
            assert_eq!(resp.data.unwrap()["status"], "pong");
        }

        #[test]
        fn driver_quit() {
            let app = TestApp { counter: 0 };
            let mut driver = HeadlessDriver::new(app, 80, 24).unwrap();
            assert!(driver.is_running());
            driver.process_request(&AgentRequest::Quit);
            assert!(!driver.is_running());
        }

        #[test]
        fn driver_render() {
            let app = TestApp { counter: 42 };
            let mut driver = HeadlessDriver::new(app, 80, 24).unwrap();
            driver.render().unwrap();
            let row = driver.row_text(0);
            assert!(row.contains("Counter: 42"));
        }

        #[test]
        fn driver_tick_advances_model() {
            let app = TestApp { counter: 0 };
            let mut driver = HeadlessDriver::new(app, 80, 24).unwrap();
            driver.tick();
            assert_eq!(driver.model().counter, 1);
            driver.tick();
            assert_eq!(driver.model().counter, 2);
        }

        #[test]
        fn driver_inject_key_event() {
            let app = TestApp { counter: 0 };
            let mut driver = HeadlessDriver::new(app, 80, 24).unwrap();
            use louie::agent::protocol::InjectedEvent;
            driver.process_request(&AgentRequest::InjectEvent {
                event: InjectedEvent::Key {
                    code: "up".into(),
                    modifiers: vec![],
                },
            });
            assert_eq!(driver.model().counter, 1);
        }

        #[test]
        fn driver_inject_quit_key() {
            let app = TestApp { counter: 0 };
            let mut driver = HeadlessDriver::new(app, 80, 24).unwrap();
            use louie::agent::protocol::InjectedEvent;
            driver.process_request(&AgentRequest::InjectEvent {
                event: InjectedEvent::Key {
                    code: "q".into(),
                    modifiers: vec![],
                },
            });
            assert!(!driver.is_running());
        }

        #[test]
        fn driver_envelope_echoes_id() {
            let app = TestApp { counter: 0 };
            let mut driver = HeadlessDriver::new(app, 80, 24).unwrap();
            let envelope = RequestEnvelope {
                id: Some("req-99".into()),
                request: AgentRequest::Ping,
            };
            let resp = driver.process_envelope(&envelope);
            assert_eq!(resp.id, Some("req-99".into()));
            assert!(resp.success);
        }

        #[test]
        fn driver_init() {
            let app = TestApp { counter: 5 };
            let mut driver = HeadlessDriver::new(app, 80, 24).unwrap();
            driver.init();
            // TestApp::init returns Command::None, so counter stays the same
            assert_eq!(driver.model().counter, 5);
        }

        #[test]
        fn driver_session_and_ontology_accessors() {
            let app = TestApp { counter: 0 };
            let driver = HeadlessDriver::new(app, 80, 24).unwrap();
            let _session = driver.session();
            let _ontology = driver.ontology();
        }
    }

    // ── Ontology Registry ────────────────────────────────────────────

    mod ontology_registry {
        use louie::ontology::registry::{OntologyRegistry, UiNode, UiTree};
        use louie::ontology::{
            AgentAction, AgentCapability, SemanticRole, WidgetSchema,
        };

        fn make_schema(name: &str, role: SemanticRole, tags: &[&str]) -> WidgetSchema {
            WidgetSchema {
                name: name.to_string(),
                description: format!("A {name} widget"),
                default_role: role,
                properties: vec![],
                actions: vec![],
                usage_hint: None,
                tags: tags.iter().map(|s| s.to_string()).collect(),
            }
        }

        #[test]
        fn register_and_get_schema() {
            let mut reg = OntologyRegistry::new();
            reg.register_schema(make_schema("MyWidget", SemanticRole::Display, &["text"]));
            assert!(reg.get_schema("MyWidget").is_some());
            assert!(reg.get_schema("Missing").is_none());
        }

        #[test]
        fn list_types() {
            let mut reg = OntologyRegistry::new();
            reg.register_schema(make_schema("A", SemanticRole::Input, &[]));
            reg.register_schema(make_schema("B", SemanticRole::Display, &[]));
            let types = reg.list_types();
            assert_eq!(types.len(), 2);
            assert!(types.contains(&"A"));
            assert!(types.contains(&"B"));
        }

        #[test]
        fn find_by_role() {
            let mut reg = OntologyRegistry::new();
            reg.register_schema(make_schema("Input1", SemanticRole::Input, &[]));
            reg.register_schema(make_schema("Input2", SemanticRole::Input, &[]));
            reg.register_schema(make_schema("Display1", SemanticRole::Display, &[]));
            let inputs = reg.find_by_role(SemanticRole::Input);
            assert_eq!(inputs.len(), 2);
        }

        #[test]
        fn search_by_tag() {
            let mut reg = OntologyRegistry::new();
            reg.register_schema(make_schema("Paragraph", SemanticRole::Display, &["text", "display"]));
            reg.register_schema(make_schema("Input", SemanticRole::Input, &["form"]));
            let results = reg.search("text");
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].name, "Paragraph");
        }

        #[test]
        fn search_by_name() {
            let mut reg = OntologyRegistry::new();
            reg.register_schema(make_schema("ProgressBar", SemanticRole::Progress, &[]));
            let results = reg.search("progress");
            assert_eq!(results.len(), 1);
        }

        #[test]
        fn search_case_insensitive() {
            let mut reg = OntologyRegistry::new();
            reg.register_schema(make_schema("MyWidget", SemanticRole::Display, &[]));
            assert_eq!(reg.search("MYWIDGET").len(), 1);
            assert_eq!(reg.search("mywidget").len(), 1);
        }

        #[test]
        fn export_catalog() {
            let mut reg = OntologyRegistry::new();
            reg.register_schema(make_schema("W1", SemanticRole::Display, &[]));
            let catalog = reg.export_catalog();
            assert!(catalog.is_object());
            assert!(catalog.get("W1").is_some());
        }

        #[test]
        fn validate_action_params_unknown_type_passes() {
            let reg = OntologyRegistry::new();
            let result = reg.validate_action_params(
                "NonExistent",
                "click",
                &serde_json::json!({}),
            );
            assert!(result.is_ok());
        }

        #[test]
        fn validate_action_params_unknown_action_passes() {
            let mut reg = OntologyRegistry::new();
            reg.register_schema(make_schema("W", SemanticRole::Display, &[]));
            let result = reg.validate_action_params("W", "unknown_action", &serde_json::json!({}));
            assert!(result.is_ok());
        }

        #[test]
        fn validate_action_params_checks_declared_actions() {
            use louie::ontology::{ActionParam, ActionParamType};

            let mut schema = make_schema("W", SemanticRole::Input, &[]);
            schema.actions.push(AgentAction {
                name: "set_value".into(),
                description: "Set a value".into(),
                params: vec![ActionParam {
                    name: "value".into(),
                    description: "The value".into(),
                    param_type: ActionParamType::String,
                    required: true,
                    default_value: None,
                }],
                returns: None,
                mutates: true,
                idempotent: true,
                shortcut: None,
            });
            let mut reg = OntologyRegistry::new();
            reg.register_schema(schema);

            // Valid params
            let ok = reg.validate_action_params("W", "set_value", &serde_json::json!({"value": "hi"}));
            assert!(ok.is_ok());

            // Missing required param
            let err = reg.validate_action_params("W", "set_value", &serde_json::json!({}));
            assert!(err.is_err());
            assert!(err.unwrap_err().contains("Missing required"));

            // Wrong type
            let err = reg.validate_action_params("W", "set_value", &serde_json::json!({"value": 42}));
            assert!(err.is_err());
        }

        // ── UI Tree ─────────────────────────────────────────────────

        #[test]
        fn tree_find_node() {
            let mut reg = OntologyRegistry::new();
            let root = UiNode::new("Container", SemanticRole::Container)
                .with_id("root")
                .with_child(UiNode::new("Button", SemanticRole::Action).with_id("btn-1"));
            reg.set_tree(UiTree::new(root));

            assert!(reg.find_node("root").is_some());
            assert!(reg.find_node("btn-1").is_some());
            assert!(reg.find_node("nonexistent").is_none());
        }

        #[test]
        fn tree_find_by_role() {
            let root = UiNode::new("Container", SemanticRole::Container)
                .with_child(UiNode::new("Btn1", SemanticRole::Action).with_id("b1"))
                .with_child(UiNode::new("Btn2", SemanticRole::Action).with_id("b2"))
                .with_child(UiNode::new("Text", SemanticRole::Display).with_id("t1"));
            let tree = UiTree::new(root);
            assert_eq!(tree.find_by_role(SemanticRole::Action).len(), 2);
            assert_eq!(tree.find_by_role(SemanticRole::Display).len(), 1);
        }

        #[test]
        fn tree_focusable_nodes() {
            let root = UiNode::new("App", SemanticRole::Container)
                .with_child(
                    UiNode::new("Input", SemanticRole::Input)
                        .with_id("i1")
                        .with_capability(AgentCapability::Focusable),
                )
                .with_child(
                    UiNode::new("Label", SemanticRole::Display)
                        .with_id("l1"),
                );
            let tree = UiTree::new(root);
            let focusable = tree.focusable_nodes();
            assert_eq!(focusable.len(), 1);
            assert_eq!(focusable[0].agent_id.as_deref(), Some("i1"));
        }

        #[test]
        fn ui_node_builder() {
            let node = UiNode::new("Test", SemanticRole::Custom(42))
                .with_id("test-1")
                .with_label("Test Node")
                .with_bounds(10, 20, 30, 40)
                .with_state(serde_json::json!({"active": true}))
                .with_capability(AgentCapability::Clickable);
            assert_eq!(node.agent_id.as_deref(), Some("test-1"));
            assert_eq!(node.label.as_deref(), Some("Test Node"));
            assert!(node.bounds.is_some());
            let b = node.bounds.unwrap();
            assert_eq!((b.x, b.y, b.width, b.height), (10, 20, 30, 40));
            assert_eq!(node.capabilities.len(), 1);
            assert_eq!(node.state["active"], true);
        }

        #[test]
        fn export_tree_null_when_empty() {
            let reg = OntologyRegistry::new();
            assert_eq!(reg.export_tree(), serde_json::Value::Null);
        }

        #[test]
        fn export_tree_json() {
            let mut reg = OntologyRegistry::new();
            let root = UiNode::new("Root", SemanticRole::Container).with_id("root");
            reg.set_tree(UiTree::new(root));
            let json = reg.export_tree();
            assert!(json.is_object());
            assert_eq!(json["root"]["widget_type"], "Root");
        }
    }

    // ── Action Validation ────────────────────────────────────────────

    mod action_validation {
        use louie::ontology::{ActionParam, ActionParamType, AgentAction};

        fn make_action(params: Vec<ActionParam>) -> AgentAction {
            AgentAction {
                name: "test_action".into(),
                description: "A test".into(),
                params,
                returns: None,
                mutates: false,
                idempotent: true,
                shortcut: None,
            }
        }

        #[test]
        fn string_type_check() {
            let action = make_action(vec![ActionParam {
                name: "s".into(),
                description: "".into(),
                param_type: ActionParamType::String,
                required: true,
                default_value: None,
            }]);
            assert!(action.validate_params(&serde_json::json!({"s": "hello"})).is_ok());
            assert!(action.validate_params(&serde_json::json!({"s": 42})).is_err());
        }

        #[test]
        fn integer_type_check() {
            let action = make_action(vec![ActionParam {
                name: "n".into(),
                description: "".into(),
                param_type: ActionParamType::Integer,
                required: true,
                default_value: None,
            }]);
            assert!(action.validate_params(&serde_json::json!({"n": 42})).is_ok());
            assert!(action.validate_params(&serde_json::json!({"n": "str"})).is_err());
        }

        #[test]
        fn float_type_check() {
            let action = make_action(vec![ActionParam {
                name: "f".into(),
                description: "".into(),
                param_type: ActionParamType::Float,
                required: true,
                default_value: None,
            }]);
            assert!(action.validate_params(&serde_json::json!({"f": 2.5})).is_ok());
            assert!(action.validate_params(&serde_json::json!({"f": 42})).is_ok()); // ints are numbers too
            assert!(action.validate_params(&serde_json::json!({"f": "str"})).is_err());
        }

        #[test]
        fn boolean_type_check() {
            let action = make_action(vec![ActionParam {
                name: "b".into(),
                description: "".into(),
                param_type: ActionParamType::Boolean,
                required: true,
                default_value: None,
            }]);
            assert!(action.validate_params(&serde_json::json!({"b": true})).is_ok());
            assert!(action.validate_params(&serde_json::json!({"b": 1})).is_err());
        }

        #[test]
        fn index_type_check() {
            let action = make_action(vec![ActionParam {
                name: "i".into(),
                description: "".into(),
                param_type: ActionParamType::Index,
                required: true,
                default_value: None,
            }]);
            assert!(action.validate_params(&serde_json::json!({"i": 0})).is_ok());
            assert!(action.validate_params(&serde_json::json!({"i": -1})).is_err());
        }

        #[test]
        fn enum_type_check() {
            let action = make_action(vec![ActionParam {
                name: "e".into(),
                description: "".into(),
                param_type: ActionParamType::Enum(vec!["a".into(), "b".into()]),
                required: true,
                default_value: None,
            }]);
            assert!(action.validate_params(&serde_json::json!({"e": "a"})).is_ok());
            assert!(action.validate_params(&serde_json::json!({"e": "c"})).is_err());
            assert!(action.validate_params(&serde_json::json!({"e": 42})).is_err());
        }

        #[test]
        fn position_type_check() {
            let action = make_action(vec![ActionParam {
                name: "p".into(),
                description: "".into(),
                param_type: ActionParamType::Position { x: true, y: true },
                required: true,
                default_value: None,
            }]);
            assert!(action.validate_params(&serde_json::json!({"p": {"x": 1, "y": 2}})).is_ok());
            assert!(action.validate_params(&serde_json::json!({"p": "bad"})).is_err());
        }

        #[test]
        fn any_type_accepts_everything() {
            let action = make_action(vec![ActionParam {
                name: "a".into(),
                description: "".into(),
                param_type: ActionParamType::Any,
                required: true,
                default_value: None,
            }]);
            assert!(action.validate_params(&serde_json::json!({"a": [1,2,3]})).is_ok());
            assert!(action.validate_params(&serde_json::json!({"a": "text"})).is_ok());
            assert!(action.validate_params(&serde_json::json!({"a": 42})).is_ok());
        }

        #[test]
        fn optional_param_missing_is_ok() {
            let action = make_action(vec![ActionParam {
                name: "opt".into(),
                description: "".into(),
                param_type: ActionParamType::String,
                required: false,
                default_value: None,
            }]);
            assert!(action.validate_params(&serde_json::json!({})).is_ok());
        }

        #[test]
        fn required_param_missing_fails() {
            let action = make_action(vec![ActionParam {
                name: "req".into(),
                description: "".into(),
                param_type: ActionParamType::String,
                required: true,
                default_value: None,
            }]);
            assert!(action.validate_params(&serde_json::json!({})).is_err());
        }
    }

    // ── Capability Names ─────────────────────────────────────────────

    mod capability_names {
        use louie::ontology::AgentCapability;

        #[test]
        fn standard_capability_names() {
            assert_eq!(AgentCapability::Focusable.name(), "focusable");
            assert_eq!(AgentCapability::Clickable.name(), "clickable");
            assert_eq!(
                AgentCapability::Scrollable {
                    vertical: true,
                    horizontal: false
                }
                .name(),
                "scrollable"
            );
            assert_eq!(
                AgentCapability::TextInput {
                    multiline: false,
                    max_length: None
                }
                .name(),
                "text-input"
            );
            assert_eq!(AgentCapability::Draggable.name(), "draggable");
            assert_eq!(AgentCapability::DropTarget.name(), "drop-target");
            assert_eq!(AgentCapability::Filterable.name(), "filterable");
            assert_eq!(AgentCapability::Searchable.name(), "searchable");
            assert_eq!(AgentCapability::Copyable.name(), "copyable");
            assert_eq!(AgentCapability::HasTooltip.name(), "has-tooltip");
        }

        #[test]
        fn custom_capability_name() {
            let cap = AgentCapability::Custom("my-feature".into());
            assert_eq!(cap.name(), "my-feature");
        }
    }

    // ── Focus Manager (extended) ─────────────────────────────────────

    mod focus_extended {
        use louie::focus::FocusManager;

        #[test]
        fn empty_ring_focus_next_is_noop() {
            let mut fm = FocusManager::new();
            fm.focus_next();
            assert_eq!(fm.focused_id(), None);
        }

        #[test]
        fn empty_ring_focus_previous_is_noop() {
            let mut fm = FocusManager::new();
            fm.focus_previous();
            assert_eq!(fm.focused_id(), None);
        }

        #[test]
        fn focus_next_wraps_around() {
            let mut fm = FocusManager::new();
            fm.register("a");
            fm.register("b");
            fm.focus_next(); // a
            fm.focus_next(); // b
            fm.focus_next(); // wraps to a
            assert_eq!(fm.focused_id(), Some("a"));
        }

        #[test]
        fn focus_previous_from_none() {
            let mut fm = FocusManager::new();
            fm.register("a");
            fm.register("b");
            fm.register("c");
            fm.focus_previous(); // starts at last: c
            assert_eq!(fm.focused_id(), Some("c"));
        }

        #[test]
        fn focus_id_not_found() {
            let mut fm = FocusManager::new();
            fm.register("a");
            assert!(!fm.focus_id("nonexistent"));
            assert_eq!(fm.focused_id(), None);
        }

        #[test]
        fn clear_resets_everything() {
            let mut fm = FocusManager::new();
            fm.register("a");
            fm.register("b");
            fm.focus_next();
            fm.clear();
            assert!(fm.is_empty());
            assert_eq!(fm.len(), 0);
            assert_eq!(fm.focused_id(), None);
        }

        #[test]
        fn ids_returns_tab_order() {
            let mut fm = FocusManager::new();
            fm.register("x");
            fm.register("y");
            fm.register("z");
            assert_eq!(fm.ids(), &["x", "y", "z"]);
        }

        #[test]
        fn len_and_is_empty() {
            let mut fm = FocusManager::new();
            assert!(fm.is_empty());
            assert_eq!(fm.len(), 0);
            fm.register("a");
            assert!(!fm.is_empty());
            assert_eq!(fm.len(), 1);
        }
    }

    // ── Overlay Stack (extended) ─────────────────────────────────────

    mod overlay_extended {
        use louie::core::rect::Rect;
        use louie::overlay::{ModalBox, Overlay, OverlayStack};

        #[test]
        fn stack_len_and_is_empty() {
            let mut stack = OverlayStack::new();
            assert!(stack.is_empty());
            assert_eq!(stack.len(), 0);
            stack.push(Overlay {
                id: "a".into(),
                area: Rect::ZERO,
                captures_focus: false,
            });
            assert!(!stack.is_empty());
            assert_eq!(stack.len(), 1);
        }

        #[test]
        fn clear_empties_stack() {
            let mut stack = OverlayStack::new();
            stack.push(Overlay {
                id: "a".into(),
                area: Rect::ZERO,
                captures_focus: false,
            });
            stack.push(Overlay {
                id: "b".into(),
                area: Rect::ZERO,
                captures_focus: true,
            });
            stack.clear();
            assert!(stack.is_empty());
        }

        #[test]
        fn remove_nonexistent_returns_false() {
            let mut stack = OverlayStack::new();
            assert!(!stack.remove("nonexistent"));
        }

        #[test]
        fn focus_capture_returns_topmost_capturing() {
            let mut stack = OverlayStack::new();
            stack.push(Overlay {
                id: "a".into(),
                area: Rect::ZERO,
                captures_focus: true,
            });
            stack.push(Overlay {
                id: "b".into(),
                area: Rect::ZERO,
                captures_focus: false,
            });
            // "a" captures, "b" doesn't — focus_capture_id is still "a"
            // Actually it's iter().rev(), so it checks from top: b (no), then a (yes)
            assert_eq!(stack.focus_capture_id(), Some("a"));
        }

        #[test]
        fn no_focus_capture_when_none_capture() {
            let mut stack = OverlayStack::new();
            stack.push(Overlay {
                id: "a".into(),
                area: Rect::ZERO,
                captures_focus: false,
            });
            assert!(!stack.has_focus_capture());
            assert_eq!(stack.focus_capture_id(), None);
        }

        #[test]
        fn iter_order() {
            let mut stack = OverlayStack::new();
            stack.push(Overlay {
                id: "first".into(),
                area: Rect::ZERO,
                captures_focus: false,
            });
            stack.push(Overlay {
                id: "second".into(),
                area: Rect::ZERO,
                captures_focus: false,
            });
            let ids: Vec<&str> = stack.iter().map(|o| o.id.as_str()).collect();
            assert_eq!(ids, &["first", "second"]);
        }

        // ── ModalBox ────────────────────────────────────────────────

        #[test]
        fn modal_inner_area() {
            let modal = ModalBox::new("Test").width_percent(50).height_percent(50);
            let parent = Rect::new(0, 0, 100, 40);
            let inner = modal.inner_area(parent);
            // 50% of 100 = 50 wide, 50% of 40 = 20 tall
            // Centered: x=25, y=10; inner is 1 cell border: x=26, y=11, w=48, h=18
            assert_eq!(inner.x, 26);
            assert_eq!(inner.y, 11);
            assert_eq!(inner.width, 48);
            assert_eq!(inner.height, 18);
        }

        #[test]
        fn modal_width_clamped_to_100() {
            let modal = ModalBox::new("Big").width_percent(200);
            let parent = Rect::new(0, 0, 80, 24);
            let inner = modal.inner_area(parent);
            // width_percent set to .min(100) = 100, so 100% of 80 = 80
            assert_eq!(inner.width, 78); // 80 - 2 for border
        }

        #[test]
        fn modal_renders_without_panic() {
            use louie::core::buffer::Buffer;
            use louie::widget::Widget;

            let modal = ModalBox::new("Dialog").width_percent(60).height_percent(40);
            let area = Rect::new(0, 0, 80, 24);
            let mut buf = Buffer::empty(area);
            modal.render(area, &mut buf);
            // Just verify it doesn't panic
        }
    }

    // ── Error Types ──────────────────────────────────────────────────

    mod error_types {
        use louie::error::{Error, Result};

        #[test]
        fn display_io() {
            let err = Error::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "gone"));
            let msg = format!("{err}");
            assert!(msg.contains("I/O error"));
            assert!(msg.contains("gone"));
        }

        #[test]
        fn display_json() {
            let json_err: std::result::Result<serde_json::Value, _> =
                serde_json::from_str("not json");
            let err = Error::Json(json_err.unwrap_err());
            let msg = format!("{err}");
            assert!(msg.contains("JSON error"));
        }

        #[test]
        fn display_protocol() {
            let msg = format!("{}", Error::Protocol("bad request".into()));
            assert!(msg.contains("Protocol error: bad request"));
        }

        #[test]
        fn display_action() {
            let msg = format!("{}", Error::Action("no such action".into()));
            assert!(msg.contains("Action error: no such action"));
        }

        #[test]
        fn display_widget_not_found() {
            let msg = format!("{}", Error::WidgetNotFound("w-1".into()));
            assert!(msg.contains("Widget not found: w-1"));
        }

        #[test]
        fn display_layout() {
            let msg = format!("{}", Error::Layout("overflow".into()));
            assert!(msg.contains("Layout error: overflow"));
        }

        #[test]
        fn from_io_error() {
            let io_err = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "broken");
            let err: Error = io_err.into();
            assert!(matches!(err, Error::Io(_)));
        }

        #[test]
        fn from_json_error() {
            let json_err = serde_json::from_str::<serde_json::Value>("!!!").unwrap_err();
            let err: Error = json_err.into();
            assert!(matches!(err, Error::Json(_)));
        }

        #[test]
        fn source_chain_io() {
            use std::error::Error as StdError;
            let err = Error::Io(std::io::Error::other("inner"));
            assert!(err.source().is_some());
        }

        #[test]
        fn source_chain_json() {
            use std::error::Error as StdError;
            let json_err = serde_json::from_str::<serde_json::Value>("{bad").unwrap_err();
            let err = Error::Json(json_err);
            assert!(err.source().is_some());
        }

        #[test]
        fn source_none_for_string_variants() {
            use std::error::Error as StdError;
            assert!(Error::Protocol("x".into()).source().is_none());
            assert!(Error::Action("x".into()).source().is_none());
            assert!(Error::WidgetNotFound("x".into()).source().is_none());
            assert!(Error::Layout("x".into()).source().is_none());
        }

        #[test]
        fn result_alias_works() {
            fn returns_ok() -> Result<i32> {
                Ok(42)
            }
            fn returns_err() -> Result<i32> {
                Err(Error::Protocol("fail".into()))
            }
            assert!(returns_ok().is_ok());
            assert!(returns_err().is_err());
        }
    }

    // ── Protocol Serde (extended) ────────────────────────────────────

    mod protocol_serde {
        use louie::agent::protocol::{AgentEvent, AgentRequest, InjectedEvent};

        #[test]
        fn all_request_variants_roundtrip() {
            let requests = vec![
                AgentRequest::Ping,
                AgentRequest::Quit,
                AgentRequest::GetTree,
                AgentRequest::QueryOntology {
                    query: Some("input".into()),
                    role: Some("display".into()),
                },
                AgentRequest::GetSchema {
                    widget_type: "List".into(),
                },
                AgentRequest::GetState {
                    agent_id: "w1".into(),
                },
                AgentRequest::ExecuteAction {
                    agent_id: "w1".into(),
                    action: "click".into(),
                    params: serde_json::json!({"x": 1}),
                },
                AgentRequest::Subscribe {
                    events: vec!["state_changed".into()],
                },
                AgentRequest::Unsubscribe {
                    events: vec!["render_update".into()],
                },
                AgentRequest::InjectEvent {
                    event: InjectedEvent::Key {
                        code: "a".into(),
                        modifiers: vec!["ctrl".into()],
                    },
                },
            ];

            for req in &requests {
                let json = serde_json::to_string(req).unwrap();
                let _parsed: AgentRequest = serde_json::from_str(&json).unwrap();
            }
        }

        #[test]
        fn injected_event_variants_roundtrip() {
            let events = vec![
                InjectedEvent::Key {
                    code: "enter".into(),
                    modifiers: vec![],
                },
                InjectedEvent::MouseClick {
                    x: 5,
                    y: 10,
                    button: "right".into(),
                },
                InjectedEvent::Paste {
                    text: "data".into(),
                },
                InjectedEvent::Resize {
                    width: 120,
                    height: 40,
                },
            ];

            for ev in &events {
                let json = serde_json::to_string(ev).unwrap();
                let _parsed: InjectedEvent = serde_json::from_str(&json).unwrap();
            }
        }

        #[test]
        fn agent_event_variants_roundtrip() {
            let events = vec![
                AgentEvent::StateChanged {
                    agent_id: "w".into(),
                    state: serde_json::json!({}),
                },
                AgentEvent::RenderUpdate {
                    tree: serde_json::json!({}),
                },
            ];

            for ev in &events {
                let json = serde_json::to_string(ev).unwrap();
                let _parsed: AgentEvent = serde_json::from_str(&json).unwrap();
            }
        }
    }
}
