//! Agent protocol and integration layer.
//!
//! This module provides the structured protocol for AI agents to connect to,
//! inspect, and control Louie applications. It includes:
//!
//! - **Protocol messages**: JSON-based request/response/event types
//! - **Agent session**: Manages agent connection lifecycle and request processing
//! - **Headless driver**: Run a Louie app without a terminal for agent-only control
//! - **RPC transport**: stdin/stdout JSON Lines protocol for embedding

pub mod driver;
pub mod protocol;
pub mod rpc;
pub mod session;

pub use driver::HeadlessDriver;
pub use protocol::{AgentEvent, AgentRequest, AgentResponse};
pub use rpc::RpcTransport;
pub use session::AgentSession;
