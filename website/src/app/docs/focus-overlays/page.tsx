export default function FocusOverlays() {
  return (
    <>
      <h1>Focus &amp; Overlays</h1>

      <h2>Focus Management</h2>
      <p>
        The <code>FocusManager</code> maintains a focus ring &mdash; an ordered list
        of focusable widget IDs supporting Tab/Shift+Tab navigation and
        programmatic focus control.
      </p>

      <pre><code>{`use hawktui::focus::FocusManager;

let mut focus = FocusManager::new();
focus.register("search-input");
focus.register("file-list");
focus.register("editor");

// Cycle focus
focus.focus_next();       // Tab
focus.focus_previous();   // Shift+Tab

// Programmatic focus (e.g., from agent action)
focus.focus_id("editor");

// Query
if focus.is_focused("editor") {
    // Render with focus style
}

let current = focus.focused_id();  // Option<&str>`}</code></pre>

      <h3>Agent Focus Control</h3>
      <p>
        Agents can focus any widget by its <code>agent_id</code> via the
        <code>execute_action</code> protocol request. Widgets with the{" "}
        <code>Focusable</code> capability appear in the focus ring automatically.
      </p>

      <h2>Overlay System</h2>
      <p>
        The <code>OverlayStack</code> manages layered overlays rendered on top of
        the main content. Overlays can capture focus, preventing interaction with
        underlying widgets.
      </p>

      <pre><code>{`use hawktui::overlay::{Overlay, OverlayStack};

let mut overlays = OverlayStack::new();

overlays.push(Overlay {
    id: "confirm-dialog".into(),
    area: dialog_rect,
    captures_focus: true,
});

// Check if any overlay captures focus
if overlays.has_focus_capture() {
    let modal_id = overlays.focus_capture_id();
    // Only this overlay receives keyboard input
}

// Remove when dismissed
overlays.remove("confirm-dialog");`}</code></pre>

      <h2>ModalBox Widget</h2>
      <p>
        The built-in <code>ModalBox</code> widget renders a centered bordered box
        with a dimmed background:
      </p>
      <pre><code>{`use hawktui::overlay::ModalBox;

let modal = ModalBox::new("Confirm Delete")
    .width_percent(50)
    .height_percent(30);

frame.render_widget(modal, frame.area());

// Get inner area for modal content
let inner = ModalBox::new("Confirm Delete")
    .inner_area(frame.area());
frame.render_widget(content, inner);`}</code></pre>

      <h2>Combining Focus + Overlays</h2>
      <p>
        When a focus-capturing overlay is active, the focus ring is temporarily
        restricted to widgets inside the overlay. When the overlay is dismissed,
        focus returns to the previous state. This is the pattern used for modal
        dialogs, command palettes, and confirmation prompts.
      </p>
    </>
  );
}
