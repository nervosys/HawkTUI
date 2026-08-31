export default function Events() {
  return (
    <>
      <h1>Events</h1>
      <p>
        Hawk TUI&apos;s event system handles keyboard, mouse, resize, paste, and focus
        events. Events are polled by the runtime and dispatched to your
        model&apos;s <code>handle_event()</code> method.
      </p>

      <h2>Event Enum</h2>
      <pre><code>{`pub enum Event {
    Key(KeyEvent),          // Keyboard input
    Mouse(MouseEvent),      // Mouse clicks, scrolls, drags
    Resize(u16, u16),       // Terminal resize (width, height)
    FocusGained,            // Terminal window focused
    FocusLost,              // Terminal window blurred
    Paste(String),          // Bracketed paste content
    Tick,                   // Timer tick (emitted every tick_rate)
}`}</code></pre>

      <h2>Keyboard Events</h2>
      <pre><code>{`pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
    pub kind: KeyEventKind,  // Press, Release, Repeat
}

// Common patterns:
if let Event::Key(key) = event {
    match key.code {
        KeyCode::Char('q') => Some(Msg::Quit),
        KeyCode::Char('s') if key.is_ctrl() => Some(Msg::Save),
        KeyCode::Up => Some(Msg::ScrollUp),
        KeyCode::Enter => Some(Msg::Submit),
        _ => None,
    }
}`}</code></pre>

      <h3>Key Codes</h3>
      <p>
        <code>Char(char)</code>, <code>F(u8)</code>, <code>Backspace</code>,{" "}
        <code>Enter</code>, <code>Tab</code>, <code>BackTab</code>,{" "}
        <code>Esc</code>, <code>Left</code>/<code>Right</code>/<code>Up</code>/<code>Down</code>,{" "}
        <code>Home</code>/<code>End</code>, <code>PageUp</code>/<code>PageDown</code>,{" "}
        <code>Insert</code>, <code>Delete</code>, <code>Null</code>
      </p>

      <h3>Modifiers</h3>
      <p>
        Bitflags: <code>NONE</code>, <code>SHIFT</code>, <code>CONTROL</code>,{" "}
        <code>ALT</code>, <code>SUPER</code>, <code>HYPER</code>, <code>META</code>.
        Check with <code>key.modifiers.contains(KeyModifiers::CONTROL)</code> or
        the shorthand <code>key.is_ctrl()</code>.
      </p>

      <h2>Mouse Events</h2>
      <pre><code>{`pub struct MouseEvent {
    pub kind: MouseEventKind,
    pub column: u16,
    pub row: u16,
    pub modifiers: KeyModifiers,
}

pub enum MouseEventKind {
    Down(MouseButton), Up(MouseButton), Drag(MouseButton),
    Moved, ScrollUp, ScrollDown, ScrollLeft, ScrollRight,
}

// Helpers:
mouse_event.is_click()       // Down(Left)
mouse_event.is_drag()        // Drag(_)
mouse_event.is_scroll()      // ScrollUp or ScrollDown
mouse_event.clicked_in(rect) // Was the click inside this Rect?`}</code></pre>

      <h2>Hit Testing</h2>
      <p>
        The <code>HitMap</code> tracks clickable regions with z-index ordering:
      </p>
      <pre><code>{`let mut hit_map = HitMap::new();
hit_map.register("button-1", button_area, 0);
hit_map.register("modal-ok", ok_area, 10);  // Higher z-index

if let Some(result) = hit_map.hit_test(mouse.column, mouse.row) {
    // result.agent_id == "modal-ok" (topmost hit)
    // result.local_position == (x_in_widget, y_in_widget)
}`}</code></pre>

      <h2>Event Filtering</h2>
      <p>
        Key release events are automatically filtered out by the runtime &mdash; only
        Press and Repeat events are dispatched. This matches expected keyboard
        behavior for TUI applications.
      </p>

      <h2>Tick Events</h2>
      <p>
        A <code>Tick</code> event is emitted every <code>tick_rate</code> interval
        (default 16ms / 60 FPS). Use ticks to advance animations, update clocks,
        or poll async results.
      </p>
    </>
  );
}
