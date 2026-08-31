export default function Security() {
  return (
    <>
      <h1>Security</h1>

      <h2>Supported Versions</h2>
      <table>
        <thead><tr><th>Version</th><th>Supported</th></tr></thead>
        <tbody>
          <tr><td>1.0.x</td><td>Yes</td></tr>
          <tr><td>&lt; 1.0</td><td>No</td></tr>
        </tbody>
      </table>

      <h2>Reporting a Vulnerability</h2>
      <div className="callout callout-warn">
        <p>
          <strong>Do not open a public issue.</strong> Report vulnerabilities via{" "}
          <a href="https://github.com/nervosys/HawkTUI/security/advisories/new">
            GitHub Security Advisories
          </a>{" "}
          or email <strong>security@nervosys.ai</strong>.
        </p>
      </div>
      <p>
        Reports are acknowledged within 48 hours. Critical fixes are released
        within 7 days.
      </p>

      <h2>Hardening Measures</h2>
      <p>
        Hawk TUI implements multiple layers of defense for the agent protocol:
      </p>
      <table>
        <thead><tr><th>Measure</th><th>Details</th></tr></thead>
        <tbody>
          <tr><td>Input size cap</td><td>1 MB per JSON line</td></tr>
          <tr><td>Rate limiting</td><td>1,000 requests/second on RPC transport</td></tr>
          <tr><td>Subscription limit</td><td>100 concurrent subscriptions</td></tr>
          <tr><td>Terminal size clamping</td><td>1&ndash;1024 for injected resize events</td></tr>
          <tr><td>Action parameter validation</td><td>Schema-checked before dispatch (INJ-2)</td></tr>
          <tr><td>Auth handshake</td><td>Optional <code>--auth-token</code> for session verification</td></tr>
          <tr><td>Binary path validation</td><td>External command paths validated (BIN-1)</td></tr>
          <tr><td>Structured logging</td><td>Sensitive fields redacted (LOG-1)</td></tr>
        </tbody>
      </table>

      <h2>Security Audit</h2>
      <p>
        The codebase was audited with the following results:
      </p>
      <ul>
        <li><strong>0 unsafe blocks</strong> in the entire codebase</li>
        <li><strong>0 known CVEs</strong> across 103 dependencies</li>
        <li>12 findings identified and remediated (2 critical, 2 high, 4 medium, 4 low)</li>
        <li>MITRE ATT&amp;CK and CMMC 2.0 Level 2 compliance documented</li>
      </ul>
      <p>
        The full audit report is available at{" "}
        <a href="https://github.com/nervosys/HawkTUI/blob/master/docs/SECURITY-AUDIT.md">
          docs/SECURITY-AUDIT.md
        </a>.
      </p>

      <h2>Dependency Auditing</h2>
      <p>
        Hawk TUI uses <code>cargo-deny</code> in CI to continuously check for:
      </p>
      <ul>
        <li>Known vulnerabilities (advisory database)</li>
        <li>License compatibility (AGPL-3.0 compliance)</li>
        <li>Duplicate dependencies</li>
        <li>Unmaintained crates</li>
      </ul>
      <pre><code>cargo deny check</code></pre>
    </>
  );
}
