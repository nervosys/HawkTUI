# Louie Security Audit Report

**Date**: 2026-03-19
**Auditor**: Automated Static Analysis + Manual Code Review
**Scope**: Full codebase (`src/`, `examples/`, `scripts/`, `tests/`, `Cargo.toml`)
**Frameworks**: CVE/NVD, MITRE ATT&CK, NIST SP 800-53 / FIPS 140-3, CMMC 2.0 (Level 2)

---

## Executive Summary

Louie is a Rust TUI framework with an agent protocol (JSON Lines over stdin/stdout). The codebase has a **small attack surface** — no network listeners, no filesystem writes, no cryptography, no authentication. The primary risk vector is a **malicious AI agent** sending crafted protocol messages to the headless server.

| Category                   | Count                     |
| -------------------------- | ------------------------- |
| Critical findings          | 2                         |
| High findings              | 2                         |
| Medium findings            | 4                         |
| Low findings               | 4                         |
| Informational              | 3                         |
| `unsafe` blocks            | **0**                     |
| Known CVEs in dependencies | **0** (cargo audit clean) |

**Overall risk**: **LOW** for its intended use case (subprocess spawned by a trusted agent). **MODERATE** if exposed to untrusted agents without additional hardening.

---

## 1. Dependency Supply Chain (CMMC 2.0 SC.L2-3.13.6, NIST SC-7)

### 1.1 cargo audit — PASS ✅

```
Scanning Cargo.lock for vulnerabilities (103 crate dependencies)
0 vulnerabilities found
```

### 1.2 Direct Dependencies

| Crate                | Version | Risk     | Notes                                   |
| -------------------- | ------- | -------- | --------------------------------------- |
| crossterm            | 0.28    | Low      | Well-maintained, terminal I/O only      |
| serde                | 1       | Low      | Industry standard, Memory safe          |
| serde_json           | 1       | Low      | Deserialization attack surface (see §3) |
| unicode-width        | 0.2     | Very Low | Pure computation                        |
| unicode-segmentation | 1.12    | Very Low | Pure computation                        |
| compact_str          | 0.8     | Low      | String optimization                     |

### 1.3 Transitive Dependencies

- **103 total crates** in Cargo.lock
- No C/FFI dependencies except via crossterm → winapi (Windows terminal API)
- No `unsafe` code in Louie itself

### 1.4 Recommendation
- Pin exact versions in Cargo.lock (already done) ✅
- Run `cargo audit` in CI pipeline
- Consider `cargo-deny` for license and source auditing

---

## 2. Memory Safety (MITRE CWE-119, CWE-125, CWE-787)

### 2.1 No unsafe Code — PASS ✅

Zero `unsafe` blocks in the entire codebase. All memory safety is guaranteed by the Rust compiler.

### 2.2 Panic Paths (Denial of Service)

| ID        | Location                             | Severity   | Description                                                                                                                                                                 |
| --------- | ------------------------------------ | ---------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **MEM-1** | `src/core/buffer.rs` L198, L205      | **MEDIUM** | `Index<(u16, u16)>` and `IndexMut<(u16, u16)>` use `.expect("position out of bounds")`. If a widget renders outside its allocated area, this panics and crashes the server. |
| **MEM-2** | `src/widget/chart.rs` L410, L442     | **LOW**    | `.unwrap()` on `ds.name.as_ref()` — but these are guarded by a prior `.filter(                                                                                              | d | d.name.is_some())` so the unwrap is safe in practice. |
| **MEM-3** | `src/bin/louie_server.rs` L313, L322 | **LOW**    | `serde_json::to_string(&resp).unwrap()` — serializing a valid `AgentResponse` to JSON cannot fail (no non-string map keys, no NaN/Inf floats). Safe in practice.            |
| **MEM-4** | `src/bin/louie_demo.rs` L129-146     | **INFO**   | Multiple `.expect()` calls on child process I/O. Acceptable for a demo binary, not for production use.                                                                      |

**MITRE ATT&CK Mapping**: T1499.004 (Application or System Exploitation for DoS)

### 2.3 Recommendations
- **MEM-1**: Replace `expect` in `Index`/`IndexMut` with safe accessors or return a default cell. This is the most impactful change.
- **MEM-2**: No action needed (guard clause present).
- **MEM-3**: No action needed (serialization of valid structs is infallible).

---

## 3. Input Validation — Agent Protocol (MITRE CWE-20, CWE-400, CWE-502)

The agent protocol is the **primary attack surface**. An untrusted agent communicates via JSON Lines on stdin.

### 3.1 FINDING: Unbounded Deserialization [CRITICAL]

**ID**: INP-1
**MITRE ATT&CK**: T1071 (Application Layer Protocol), T1190 (Exploit Public-Facing Application)
**CWE**: CWE-400 (Uncontrolled Resource Consumption), CWE-502 (Deserialization of Untrusted Data)
**CMMC**: SC.L2-3.13.6 (Network Communication Traffic), SI.L2-3.14.2 (Flaw Remediation)

**Description**: `serde_json::from_str(trimmed)` in `louie_server.rs` (L305) and `rpc.rs` (L75) deserializes arbitrary JSON with no size limits. An attacker can:

1. Send a single line containing a multi-gigabyte JSON string → OOM kill
2. Send deeply nested JSON → stack overflow during parsing
3. Send a `Paste` event with a multi-gigabyte `text` field → OOM
4. Send `Subscribe` with millions of event type strings → HashSet memory exhaustion

**Locations**:
- `src/bin/louie_server.rs:305` — `serde_json::from_str(trimmed)`
- `src/agent/rpc.rs:75` — `serde_json::from_str(trimmed)`

**Remediation**:
```rust
// Add before deserialization:
const MAX_LINE_BYTES: usize = 1_048_576; // 1 MB
if trimmed.len() > MAX_LINE_BYTES {
    let resp = AgentResponse::err("Request too large (max 1MB)");
    // ... write response and continue
    continue;
}
```

### 3.2 FINDING: No Terminal Size Validation [CRITICAL]

**ID**: INP-2
**CWE**: CWE-400 (Uncontrolled Resource Consumption)
**CMMC**: SC.L2-3.13.6

**Description**: `louie_server.rs` accepts `--width` and `--height` as `u16` values (0–65535). A `Buffer::empty(Rect)` allocates `width * height` cells. At max values: 65535 × 65535 = 4,294,836,225 cells × ~24 bytes/cell = **~103 GB** allocated.

Additionally, `InjectedEvent::Resize { width, height }` allows an agent to resize the terminal at runtime with no bounds check.

**Location**: `src/bin/louie_server.rs:225-228` (CLI), `src/agent/session.rs:201` (resize event)

**Remediation**:
```rust
const MAX_WIDTH: u16 = 1024;
const MAX_HEIGHT: u16 = 512;

// In CLI:
args.width = args.width.min(MAX_WIDTH).max(1);
args.height = args.height.min(MAX_HEIGHT).max(1);

// In convert_injected_event for Resize:
let width = (*width).min(MAX_WIDTH).max(1);
let height = (*height).min(MAX_HEIGHT).max(1);
```

### 3.3 FINDING: Unbounded Subscription Set [HIGH]

**ID**: INP-3
**CWE**: CWE-400

**Description**: `AgentSession::subscriptions` is a `HashSet<String>` that grows without limit. An agent can send `Subscribe { events: ["a"*10000, "b"*10000, ...] }` repeatedly to exhaust memory.

**Location**: `src/agent/session.rs:142-145`

**Remediation**: Cap subscriptions at 100 entries and validate event type names against a known set.

### 3.4 FINDING: No Rate Limiting on RPC Loop [HIGH]

**ID**: INP-4
**CWE**: CWE-400
**MITRE ATT&CK**: T1499.003 (Application Exhaustion Flood)

**Description**: The stdin read loop processes requests as fast as they arrive with no rate limit. A malicious agent can flood the server with thousands of requests per second (e.g., `inject_event` key presses), causing CPU exhaustion and potential log memory growth.

**Locations**: `src/bin/louie_server.rs:299-330`, `src/agent/rpc.rs:69-120`

**Remediation**: Add a configurable rate limit (e.g., max 1000 requests/second) or per-request backpressure.

---

## 4. Injection Attacks (MITRE CWE-77, CWE-78, CWE-79)

### 4.1 Command Injection — NOT APPLICABLE ✅

The codebase does not execute shell commands, call `std::process::Command` with user input, or perform any system calls with agent-provided data. The only `Command::new()` usage is in `louie_demo.rs` with a hardcoded path to `louie-server`.

### 4.2 SQL Injection — NOT APPLICABLE ✅

No database access.

### 4.3 Cross-Site Scripting — NOT APPLICABLE ✅

No web output. All rendering is to an in-memory terminal buffer.

### 4.4 FINDING: Paste Injection [MEDIUM]

**ID**: INJ-1
**CWE**: CWE-94 (Code Injection via Paste)

**Description**: `InjectedEvent::Paste { text }` is passed directly to the application model via `Event::Paste(text.clone())`. If the model uses paste content to construct shell commands, file paths, or other sensitive operations, this becomes an injection vector.

**Location**: `src/agent/session.rs:200`

**Note**: This is an application-level concern, not a framework bug. The framework correctly passes the paste content verbatim — it's up to the consumer model to sanitize.

**Recommendation**: Document in the protocol specification that applications MUST validate paste content before using it in sensitive operations.

### 4.5 FINDING: ExecuteAction Parameter Passthrough [MEDIUM]

**ID**: INJ-2
**CWE**: CWE-20

**Description**: `ExecuteAction { params: serde_json::Value }` passes an arbitrary JSON value to the model's action handler. The framework does no validation of the `params` against the widget's declared schema constraints.

**Location**: `src/agent/rpc.rs:96-102`, `src/agent/driver.rs:89-96`

**Recommendation**: Add optional schema validation for action params against the widget's declared `PropertySchema` constraints before dispatch.

---

## 5. Binary Path Resolution (MITRE CWE-426, CWE-427)

### 5.1 FINDING: Relative Path Binary Lookup [MEDIUM]

**ID**: BIN-1
**CWE**: CWE-427 (Uncontrolled Search Path Element)
**MITRE ATT&CK**: T1574.001 (Hijack Execution Flow: DLL Search Order Hijacking — analogous for binaries)

**Description**: `louie_demo.rs` resolves the server binary via relative paths `target/release/louie-server` and `target/debug/louie-server`. If the CWD is manipulated or a malicious binary is placed at that path, it would be executed.

**Location**: `src/bin/louie_demo.rs:86-107`

**Severity**: **MEDIUM** — This binary is a demo tool, not production infrastructure. An attacker with filesystem write access already has broader compromise.

**Recommendation**: Use `std::env::current_exe()` to resolve the binary directory, or accept the server path as a CLI argument.

---

## 6. Cryptography (NIST FIPS 140-3, CMMC 2.0 SC.L2-3.13.11)

**NOT APPLICABLE** — Louie performs no cryptographic operations, stores no secrets, and transmits no data over networks. The stdin/stdout protocol is plaintext by design (local subprocess pipe).

If deployed over a network (SSH tunnel, TLS proxy), encryption would be the transport layer's responsibility, not the framework's.

---

## 7. Authentication & Access Control (CMMC 2.0 AC.L2-3.1.1, IA.L2-3.5.1)

### 7.1 FINDING: No Agent Authentication [LOW]

**ID**: AUTH-1
**CWE**: CWE-306 (Missing Authentication)
**CMMC**: IA.L2-3.5.1 (Identification), AC.L2-3.1.1 (Authorized Access Control)

**Description**: Any process that can write to louie-server's stdin can send protocol commands. There is no authentication, authorization, or session token mechanism.

**Severity**: **LOW** — The server is designed as a local subprocess. stdin/stdout pipes inherit the OS process security context. Only the parent process (the agent) has pipe access.

**Recommendation**: For network-exposed deployments, add a challenge-response handshake or shared secret at connection time. For subprocess mode, the OS process isolation is sufficient.

### 7.2 No RBAC / Least Privilege — INFORMATIONAL

All protocol commands are available to any connected agent. There is no role-based access control (e.g., read-only agent vs. full-control agent).

**Recommendation**: For multi-agent scenarios, consider adding permission levels (observe-only, interact, admin).

---

## 8. Logging & Monitoring (CMMC 2.0 AU.L2-3.3.1, NIST AU-2)

### 8.1 FINDING: No Structured Security Logging [LOW]

**ID**: LOG-1
**CMMC**: AU.L2-3.3.1 (System Auditing)

**Description**: The server logs to stderr with `eprintln!()` — no structured logging, no timestamps, no request correlation, no audit trail.

**Recommendations**:
- Add timestamps to all log messages
- Log all incoming request types (without sensitive payload content)
- Log failed parse attempts (potential attack indicators)
- Consider `tracing` crate for structured logging

---

## 9. Data Protection (CMMC 2.0 MP.L2-3.8.3, SC.L2-3.13.16)

### 9.1 Data at Rest — NOT APPLICABLE ✅

Louie stores no data to disk. All state is in-memory and ephemeral.

### 9.2 Data in Transit — LOW RISK

stdin/stdout pipes are local OS constructs — data never traverses a network. If an integrator tunnels the protocol over TCP/SSH, they are responsible for encryption.

### 9.3 FINDING: No Private Data in Source [CONFIRMED ✅]

**ID**: DATA-1

Full codebase scan confirmed:
- No API keys, secrets, tokens, or credentials
- No email addresses or personal information
- No hardcoded IPs or internal hostnames
- `target/` and `reference/` directories excluded via `.gitignore`
- `Cargo.toml` authors field contains only "Nervosys" (company name)

---

## 10. MITRE ATT&CK Mapping

| Technique                   | ID        | Applicability                 | Risk               |
| --------------------------- | --------- | ----------------------------- | ------------------ |
| Application Layer Protocol  | T1071     | JSON Lines over stdin         | Low (local pipe)   |
| Exploit Public-Facing App   | T1190     | N/A (not network-facing)      | N/A                |
| DoS: Application Exhaustion | T1499.003 | Unbounded deserialization     | High               |
| DoS: Endpoint Exhaustion    | T1499.004 | Panic in Buffer Index         | Medium             |
| Hijack Execution Flow       | T1574.001 | Relative binary path          | Medium (demo only) |
| Input Capture               | T1056     | Agent captures all keystrokes | By Design          |
| Data from Local System      | T1005     | Agent reads UI state          | By Design          |

---

## 11. CMMC 2.0 Level 2 Controls Checklist

| Domain        | Control                   | Status    | Notes                                 |
| ------------- | ------------------------- | --------- | ------------------------------------- |
| AC.L2-3.1.1   | Authorized Access Control | ⚠️ PARTIAL | OS-level process isolation only       |
| AU.L2-3.3.1   | System Auditing           | ❌ GAP     | No structured audit logging           |
| IA.L2-3.5.1   | Identification            | ⚠️ PARTIAL | No agent authentication               |
| SC.L2-3.13.6  | Network Traffic Control   | ✅ PASS    | Local pipe, no network                |
| SC.L2-3.13.11 | Cryptographic Protection  | ✅ N/A     | No crypto operations                  |
| SC.L2-3.13.16 | Data at Rest Protection   | ✅ N/A     | No persistent storage                 |
| SI.L2-3.14.1  | Flaw Identification       | ✅ PASS    | 0 CVEs in deps                        |
| SI.L2-3.14.2  | Flaw Remediation          | ⚠️ PARTIAL | Findings INP-1 through INP-4 open     |
| SI.L2-3.14.6  | Security Alerts           | ⚠️ PARTIAL | cargo audit passes, no CI integration |
| MP.L2-3.8.3   | Media Protection          | ✅ N/A     | No storage media                      |

---

## 12. Remediation Priority

| Priority | ID     | Finding                                         | Effort |
| -------- | ------ | ----------------------------------------------- | ------ |
| **P0**   | INP-1  | Add max request size limit (1 MB)               | 15 min |
| **P0**   | INP-2  | Validate terminal size bounds (max 1024×512)    | 15 min |
| **P1**   | INP-3  | Cap subscription set size                       | 10 min |
| **P1**   | INP-4  | Add configurable rate limit                     | 30 min |
| **P1**   | MEM-1  | Replace panic in Buffer Index with safe default | 20 min |
| **P2**   | INJ-2  | Add schema validation for action params         | 2 hr   |
| **P2**   | LOG-1  | Add structured logging with timestamps          | 1 hr   |
| **P2**   | AUTH-1 | Add optional shared-secret handshake            | 2 hr   |
| **P3**   | BIN-1  | Use absolute path resolution in demo            | 10 min |
| **P3**   | INJ-1  | Document paste sanitization responsibility      | 10 min |

---

## 13. Positive Security Findings

- **Zero `unsafe` code** — Entire codebase is memory-safe by Rust guarantees
- **Zero known CVEs** — All 103 dependencies clean per RustSec advisory DB
- **No network exposure** — stdin/stdout only, no TCP listeners
- **No filesystem writes** — Zero file I/O in library or server
- **No shell execution** — No `Command::new()` with user input
- **Proper error handling** — Deserialization errors return JSON error responses, don't panic
- **Bounded arithmetic** — All `Rect` methods use `saturating_add`/`saturating_sub`
- **Type-safe protocol** — Serde tagged enum prevents variant confusion
- **No dynamic code execution** — No `eval()`, no plugin loading
- **Minimal dependency tree** — 6 direct deps, all well-audited crates

---

*Report generated 2026-03-19. Next audit recommended after: major dependency update, network transport addition, or authentication system integration.*
