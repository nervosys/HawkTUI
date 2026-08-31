export default function Footer() {
  return (
    <footer className="footer">
      <p style={{ fontFamily: '"Cascadia Code", "Fira Code", monospace', fontSize: '0.75rem', letterSpacing: '0.1em', marginBottom: '8px', color: 'rgba(0, 255, 200, 0.3)' }}>
        NERVOSYS // AGENTIC SOFTWARE
      </p>
      <p>
        Hawk TUI is licensed under AGPL-3.0-or-later. Commercial licenses available
        from <a href="https://nervosys.ai">NERVOSYS</a>.
      </p>
      <p>&copy; {new Date().getFullYear()} Nervosys. All rights reserved.</p>
    </footer>
  );
}
