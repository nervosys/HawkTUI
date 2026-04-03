export default function Animation() {
  return (
    <>
      <h1>Animation</h1>
      <p>
        Louie includes a full animation engine with tweens, springs, 25 easing
        curves, and timeline sequencing. Drive animations from the Tick event in
        your update loop.
      </p>

      <h2>Tweens</h2>
      <p>
        Interpolate between two values over a duration with an easing curve:
      </p>
      <pre><code>{`use louie::animation::{Tween, Easing, Animation};
use std::time::Duration;

let mut tween = Tween::new(0.0, 1.0, Duration::from_millis(300))
    .easing(Easing::EaseInOutCubic)
    .delay(Duration::from_millis(100));

// In your update loop:
let dt = Duration::from_millis(16);
let still_running = tween.tick(dt);  // returns false when finished
let current = tween.value();          // 0.0 → 1.0`}</code></pre>

      <h3>Repeat Modes</h3>
      <pre><code>{`use louie::animation::RepeatMode;

Tween::new(0.0, 1.0, Duration::from_millis(500))
    .repeat(RepeatMode::Loop)       // Repeat forever
    .repeat(RepeatMode::PingPong)   // Bounce back and forth
    .repeat(RepeatMode::Count(3))   // Repeat 3 times`}</code></pre>

      <h2>Springs</h2>
      <p>Physics-based animation with configurable stiffness and damping:</p>
      <pre><code>{`use louie::animation::Spring;

let mut spring = Spring::new(0.0, 100.0)  // initial, target
    .stiffness(170.0)
    .damping(26.0)
    .mass(1.0);

// Retarget dynamically:
spring.set_target(200.0);

// In your update loop:
spring.tick(dt);
let current = spring.value();  // Converges to target`}</code></pre>

      <h2>25 Easing Curves</h2>
      <table>
        <thead><tr><th>Category</th><th>Easings</th></tr></thead>
        <tbody>
          <tr><td>Linear</td><td><code>Linear</code></td></tr>
          <tr><td>Standard</td><td><code>EaseIn</code>, <code>EaseOut</code>, <code>EaseInOut</code></td></tr>
          <tr><td>Quadratic</td><td><code>EaseInQuad</code>, <code>EaseOutQuad</code>, <code>EaseInOutQuad</code></td></tr>
          <tr><td>Cubic</td><td><code>EaseInCubic</code>, <code>EaseOutCubic</code>, <code>EaseInOutCubic</code></td></tr>
          <tr><td>Quartic</td><td><code>EaseInQuart</code>, <code>EaseOutQuart</code>, <code>EaseInOutQuart</code></td></tr>
          <tr><td>Exponential</td><td><code>EaseInExpo</code>, <code>EaseOutExpo</code>, <code>EaseInOutExpo</code></td></tr>
          <tr><td>Back</td><td><code>EaseInBack</code>, <code>EaseOutBack</code>, <code>EaseInOutBack</code></td></tr>
          <tr><td>Elastic</td><td><code>EaseInElastic</code>, <code>EaseOutElastic</code>, <code>EaseInOutElastic</code></td></tr>
          <tr><td>Bounce</td><td><code>EaseInBounce</code>, <code>EaseOutBounce</code>, <code>EaseInOutBounce</code></td></tr>
          <tr><td>Custom</td><td><code>CubicBezier(x1, y1, x2, y2)</code></td></tr>
        </tbody>
      </table>

      <h2>Timelines</h2>
      <p>Compose multiple animations into sequential or parallel tracks:</p>
      <pre><code>{`use louie::animation::Timeline;

// Sequential: one after another
let mut timeline = Timeline::sequential()
    .add_labeled("fade_in", Tween::new(0.0, 1.0, Duration::from_millis(200)))
    .add_labeled("slide", Tween::new(0.0, 50.0, Duration::from_millis(300)));

timeline.tick(dt);
let opacity = timeline.get_value("fade_in");
let offset = timeline.get_value("slide");

// Parallel: all at once
let mut parallel = Timeline::parallel()
    .add_labeled("x", Tween::new(0.0, 100.0, Duration::from_millis(500)))
    .add_labeled("y", Tween::new(0.0, 50.0, Duration::from_millis(500)));`}</code></pre>

      <h2>Animator Helper</h2>
      <p>
        The <code>Animator</code> type manages multiple named animations per
        widget:
      </p>
      <pre><code>{`use louie::animation::Animator;

let mut animator = Animator::new();
animator.add("progress", Tween::new(0.0, 1.0, Duration::from_millis(800)));

// In update loop:
animator.tick();
if let Some(val) = animator.get("progress") {
    // Use val for rendering
}
let still_animating = animator.is_animating();`}</code></pre>
    </>
  );
}
