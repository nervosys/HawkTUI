//! Animation engine with tweens, springs, easing functions, and timelines.
//!
//! Provides first-class animation primitives that widgets can use to create
//! smooth transitions, physics-based motion, and choreographed sequences.

use std::time::{Duration, Instant};

/// Common trait for all animation types.
pub trait Animation {
    /// Advance the animation by the given delta time.
    /// Returns `true` if the animation is still running.
    fn tick(&mut self, dt: Duration) -> bool;

    /// Current interpolated value (0.0 to 1.0 for progress, or arbitrary for springs).
    fn value(&self) -> f64;

    /// Whether the animation has completed.
    fn is_finished(&self) -> bool;

    /// Reset the animation to its initial state.
    fn reset(&mut self);
}

// ── Easing Functions ─────────────────────────────────────────────────────

/// Standard easing functions for smooth animations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Easing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    EaseInQuart,
    EaseOutQuart,
    EaseInOutQuart,
    EaseInExpo,
    EaseOutExpo,
    EaseInOutExpo,
    EaseInBack,
    EaseOutBack,
    EaseInOutBack,
    EaseInElastic,
    EaseOutElastic,
    EaseInOutElastic,
    EaseInBounce,
    EaseOutBounce,
    EaseInOutBounce,
    /// Custom cubic bezier (x1, y1, x2, y2).
    CubicBezier(f64, f64, f64, f64),
}

impl Easing {
    /// Apply this easing function to a linear progress value `t` in [0, 1].
    pub fn apply(&self, t: f64) -> f64 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Self::Linear => t,
            Self::EaseIn => t * t * t,
            Self::EaseOut => 1.0 - (1.0 - t).powi(3),
            Self::EaseInOut => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }
            Self::EaseInQuad => t * t,
            Self::EaseOutQuad => 1.0 - (1.0 - t) * (1.0 - t),
            Self::EaseInOutQuad => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
            Self::EaseInCubic => t * t * t,
            Self::EaseOutCubic => 1.0 - (1.0 - t).powi(3),
            Self::EaseInOutCubic => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }
            Self::EaseInQuart => t * t * t * t,
            Self::EaseOutQuart => 1.0 - (1.0 - t).powi(4),
            Self::EaseInOutQuart => {
                if t < 0.5 {
                    8.0 * t * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(4) / 2.0
                }
            }
            Self::EaseInExpo => {
                if t == 0.0 {
                    0.0
                } else {
                    (2.0f64).powf(10.0 * t - 10.0)
                }
            }
            Self::EaseOutExpo => {
                if t == 1.0 {
                    1.0
                } else {
                    1.0 - (2.0f64).powf(-10.0 * t)
                }
            }
            Self::EaseInOutExpo => {
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else if t < 0.5 {
                    (2.0f64).powf(20.0 * t - 10.0) / 2.0
                } else {
                    (2.0 - (2.0f64).powf(-20.0 * t + 10.0)) / 2.0
                }
            }
            Self::EaseInBack => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                c3 * t * t * t - c1 * t * t
            }
            Self::EaseOutBack => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
            }
            Self::EaseInOutBack => {
                let c1 = 1.70158;
                let c2 = c1 * 1.525;
                if t < 0.5 {
                    ((2.0 * t).powi(2) * ((c2 + 1.0) * 2.0 * t - c2)) / 2.0
                } else {
                    ((2.0 * t - 2.0).powi(2) * ((c2 + 1.0) * (2.0 * t - 2.0) + c2) + 2.0) / 2.0
                }
            }
            Self::EaseInElastic => {
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    let c4 = (2.0 * std::f64::consts::PI) / 3.0;
                    -(2.0f64).powf(10.0 * t - 10.0) * ((10.0 * t - 10.75) * c4).sin()
                }
            }
            Self::EaseOutElastic => {
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    let c4 = (2.0 * std::f64::consts::PI) / 3.0;
                    (2.0f64).powf(-10.0 * t) * ((10.0 * t - 0.75) * c4).sin() + 1.0
                }
            }
            Self::EaseInOutElastic => {
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    let c5 = (2.0 * std::f64::consts::PI) / 4.5;
                    if t < 0.5 {
                        -((2.0f64).powf(20.0 * t - 10.0) * ((20.0 * t - 11.125) * c5).sin()) / 2.0
                    } else {
                        ((2.0f64).powf(-20.0 * t + 10.0) * ((20.0 * t - 11.125) * c5).sin()) / 2.0
                            + 1.0
                    }
                }
            }
            Self::EaseInBounce => 1.0 - Self::EaseOutBounce.apply(1.0 - t),
            Self::EaseOutBounce => {
                let n1 = 7.5625;
                let d1 = 2.75;
                if t < 1.0 / d1 {
                    n1 * t * t
                } else if t < 2.0 / d1 {
                    let t = t - 1.5 / d1;
                    n1 * t * t + 0.75
                } else if t < 2.5 / d1 {
                    let t = t - 2.25 / d1;
                    n1 * t * t + 0.9375
                } else {
                    let t = t - 2.625 / d1;
                    n1 * t * t + 0.984375
                }
            }
            Self::EaseInOutBounce => {
                if t < 0.5 {
                    (1.0 - Self::EaseOutBounce.apply(1.0 - 2.0 * t)) / 2.0
                } else {
                    (1.0 + Self::EaseOutBounce.apply(2.0 * t - 1.0)) / 2.0
                }
            }
            Self::CubicBezier(x1, y1, x2, y2) => cubic_bezier_at(t, *x1, *y1, *x2, *y2),
        }
    }
}

/// Approximate cubic bezier value using binary search on t.
fn cubic_bezier_at(t: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    // Binary search for the t parameter along the x-axis
    let mut lo = 0.0f64;
    let mut hi = 1.0f64;
    for _ in 0..20 {
        let mid = (lo + hi) / 2.0;
        let x = bezier_component(mid, x1, x2);
        if x < t {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    bezier_component((lo + hi) / 2.0, y1, y2)
}

fn bezier_component(t: f64, p1: f64, p2: f64) -> f64 {
    let t2 = t * t;
    let t3 = t2 * t;
    3.0 * (1.0 - t) * (1.0 - t) * t * p1 + 3.0 * (1.0 - t) * t2 * p2 + t3
}

// ── Tween ────────────────────────────────────────────────────────────────

/// A time-based interpolation from one value to another.
#[derive(Debug, Clone)]
pub struct Tween {
    from: f64,
    to: f64,
    duration: Duration,
    elapsed: Duration,
    easing: Easing,
    delay: Duration,
    repeat: RepeatMode,
    current_value: f64,
}

/// How a tween should repeat.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeatMode {
    /// Play once and stop.
    Once,
    /// Loop from start when complete.
    Loop,
    /// Reverse direction at each end (ping-pong).
    PingPong,
    /// Repeat a fixed number of times.
    Count(u32),
}

impl Tween {
    /// Create a new tween that interpolates from `from` to `to` over `duration`.
    pub fn new(from: f64, to: f64, duration: Duration) -> Self {
        Self {
            from,
            to,
            duration,
            elapsed: Duration::ZERO,
            easing: Easing::Linear,
            delay: Duration::ZERO,
            repeat: RepeatMode::Once,
            current_value: from,
        }
    }

    /// Set the easing function (default: `Linear`).
    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Set a delay before the tween starts.
    pub fn delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    /// Set the repeat mode (default: `Once`).
    pub fn repeat(mut self, mode: RepeatMode) -> Self {
        self.repeat = mode;
        self
    }

    /// Interpolate a value between `from` and `to` using the current progress.
    pub fn lerp(from: f64, to: f64, t: f64) -> f64 {
        from + (to - from) * t
    }
}

impl Animation for Tween {
    fn tick(&mut self, dt: Duration) -> bool {
        self.elapsed += dt;

        if self.elapsed < self.delay {
            return true;
        }

        let active_elapsed = self.elapsed - self.delay;
        let dur_secs = self.duration.as_secs_f64();
        if dur_secs == 0.0 {
            self.current_value = self.to;
            return false;
        }

        let raw_progress = active_elapsed.as_secs_f64() / dur_secs;

        let (progress, running) = match self.repeat {
            RepeatMode::Once => {
                let p = raw_progress.min(1.0);
                (p, p < 1.0)
            }
            RepeatMode::Loop => {
                let p = raw_progress % 1.0;
                (p, true)
            }
            RepeatMode::PingPong => {
                let cycle = raw_progress % 2.0;
                let p = if cycle > 1.0 { 2.0 - cycle } else { cycle };
                (p, true)
            }
            RepeatMode::Count(n) => {
                let total_dur = dur_secs * n as f64;
                if active_elapsed.as_secs_f64() >= total_dur {
                    (1.0, false)
                } else {
                    (raw_progress % 1.0, true)
                }
            }
        };

        let eased = self.easing.apply(progress);
        self.current_value = Tween::lerp(self.from, self.to, eased);
        running
    }

    fn value(&self) -> f64 {
        self.current_value
    }

    fn is_finished(&self) -> bool {
        match self.repeat {
            RepeatMode::Once => {
                (self.elapsed.saturating_sub(self.delay)).as_secs_f64()
                    >= self.duration.as_secs_f64()
            }
            RepeatMode::Loop | RepeatMode::PingPong => false,
            RepeatMode::Count(n) => {
                (self.elapsed.saturating_sub(self.delay)).as_secs_f64()
                    >= self.duration.as_secs_f64() * n as f64
            }
        }
    }

    fn reset(&mut self) {
        self.elapsed = Duration::ZERO;
        self.current_value = self.from;
    }
}

// ── Spring Physics ───────────────────────────────────────────────────────

/// A spring-based animation for natural, physics-driven motion.
///
/// Uses a critically-damped spring model. Configure `stiffness`, `damping`,
/// and `mass` to control the feel.
#[derive(Debug, Clone)]
pub struct Spring {
    /// Target value the spring is moving toward.
    pub target: f64,
    /// Current position.
    position: f64,
    /// Current velocity.
    velocity: f64,
    /// Spring stiffness (k). Higher = snappier.
    pub stiffness: f64,
    /// Damping coefficient. Higher = less oscillation.
    pub damping: f64,
    /// Mass of the object. Higher = more sluggish.
    pub mass: f64,
    /// Velocity threshold below which the spring is considered at rest.
    rest_threshold: f64,
}

impl Spring {
    /// Create a spring that starts at `initial` and settles toward `target`.
    pub fn new(initial: f64, target: f64) -> Self {
        Self {
            target,
            position: initial,
            velocity: 0.0,
            stiffness: 170.0,
            damping: 26.0,
            mass: 1.0,
            rest_threshold: 0.01,
        }
    }

    /// Set spring stiffness (default: 170). Higher = snappier.
    pub fn stiffness(mut self, stiffness: f64) -> Self {
        self.stiffness = stiffness;
        self
    }

    /// Set damping coefficient (default: 26). Higher = less oscillation.
    pub fn damping(mut self, damping: f64) -> Self {
        self.damping = damping;
        self
    }

    /// Set mass (default: 1). Higher = more sluggish.
    pub fn mass(mut self, mass: f64) -> Self {
        self.mass = mass;
        self
    }

    /// Set a new target (animates toward it).
    pub fn set_target(&mut self, target: f64) {
        self.target = target;
    }
}

impl Animation for Spring {
    fn tick(&mut self, dt: Duration) -> bool {
        let dt = dt.as_secs_f64();
        if dt == 0.0 {
            return !self.is_finished();
        }

        // Spring force: F = -k * (x - target) - d * v
        let displacement = self.position - self.target;
        let spring_force = -self.stiffness * displacement;
        let damping_force = -self.damping * self.velocity;
        let acceleration = (spring_force + damping_force) / self.mass;

        self.velocity += acceleration * dt;
        self.position += self.velocity * dt;

        !self.is_finished()
    }

    fn value(&self) -> f64 {
        self.position
    }

    fn is_finished(&self) -> bool {
        let displacement = (self.position - self.target).abs();
        displacement < self.rest_threshold && self.velocity.abs() < self.rest_threshold
    }

    fn reset(&mut self) {
        self.velocity = 0.0;
    }
}

// ── Timeline ─────────────────────────────────────────────────────────────

/// A sequence of animations that play in order or in parallel.
#[derive(Debug)]
pub struct Timeline {
    entries: Vec<TimelineEntry>,
    mode: TimelineMode,
    current_index: usize,
    finished: bool,
}

#[derive(Debug)]
struct TimelineEntry {
    animation: Box<dyn Animation>,
    label: Option<String>,
}

impl std::fmt::Debug for dyn Animation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Animation")
    }
}

/// How timeline entries are played.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimelineMode {
    /// Play each animation one after another.
    Sequential,
    /// Play all animations simultaneously.
    Parallel,
}

impl Timeline {
    /// Create a timeline that plays animations one after another.
    pub fn sequential() -> Self {
        Self {
            entries: Vec::new(),
            mode: TimelineMode::Sequential,
            current_index: 0,
            finished: false,
        }
    }

    /// Create a timeline that plays all animations simultaneously.
    pub fn parallel() -> Self {
        Self {
            entries: Vec::new(),
            mode: TimelineMode::Parallel,
            current_index: 0,
            finished: false,
        }
    }

    #[allow(clippy::should_implement_trait)]
    /// Append an animation to this timeline (builder pattern).
    pub fn add(mut self, animation: impl Animation + 'static) -> Self {
        self.entries.push(TimelineEntry {
            animation: Box::new(animation),
            label: None,
        });
        self
    }

    pub fn add_labeled(
        mut self,
        label: impl Into<String>,
        animation: impl Animation + 'static,
    ) -> Self {
        self.entries.push(TimelineEntry {
            animation: Box::new(animation),
            label: Some(label.into()),
        });
        self
    }

    /// Get the value of a labeled animation.
    pub fn get_value(&self, label: &str) -> Option<f64> {
        self.entries
            .iter()
            .find(|e| e.label.as_deref() == Some(label))
            .map(|e| e.animation.value())
    }
}

impl Animation for Timeline {
    fn tick(&mut self, dt: Duration) -> bool {
        if self.finished || self.entries.is_empty() {
            return false;
        }

        match self.mode {
            TimelineMode::Sequential => {
                if self.current_index < self.entries.len() {
                    let running = self.entries[self.current_index].animation.tick(dt);
                    if !running {
                        self.current_index += 1;
                        if self.current_index >= self.entries.len() {
                            self.finished = true;
                            return false;
                        }
                    }
                    true
                } else {
                    self.finished = true;
                    false
                }
            }
            TimelineMode::Parallel => {
                let mut any_running = false;
                for entry in &mut self.entries {
                    if entry.animation.tick(dt) {
                        any_running = true;
                    }
                }
                if !any_running {
                    self.finished = true;
                }
                any_running
            }
        }
    }

    fn value(&self) -> f64 {
        match self.mode {
            TimelineMode::Sequential => {
                if self.current_index < self.entries.len() {
                    self.entries[self.current_index].animation.value()
                } else if let Some(last) = self.entries.last() {
                    last.animation.value()
                } else {
                    0.0
                }
            }
            TimelineMode::Parallel => {
                // For parallel, return average progress
                if self.entries.is_empty() {
                    0.0
                } else {
                    let sum: f64 = self.entries.iter().map(|e| e.animation.value()).sum();
                    sum / self.entries.len() as f64
                }
            }
        }
    }

    fn is_finished(&self) -> bool {
        self.finished
    }

    fn reset(&mut self) {
        self.current_index = 0;
        self.finished = false;
        for entry in &mut self.entries {
            entry.animation.reset();
        }
    }
}

// ── Animator (per-widget helper) ─────────────────────────────────────────

/// Manages multiple named animations for a widget.
#[derive(Debug, Default)]
pub struct Animator {
    animations: Vec<(String, Box<dyn Animation>)>,
    last_tick: Option<Instant>,
}

impl Animator {
    /// Create a new empty animator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a named animation.
    pub fn add(&mut self, name: impl Into<String>, animation: impl Animation + 'static) {
        self.animations.push((name.into(), Box::new(animation)));
    }

    /// Get the current value of a named animation.
    pub fn get(&self, name: &str) -> Option<f64> {
        self.animations
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, a)| a.value())
    }

    /// Tick all animations. Returns true if any are still running.
    pub fn tick(&mut self) -> bool {
        let now = Instant::now();
        let dt = self
            .last_tick
            .map(|last| now.duration_since(last))
            .unwrap_or(Duration::from_millis(16));
        self.last_tick = Some(now);

        let mut any_running = false;
        self.animations.retain_mut(|(_, anim)| {
            let running = anim.tick(dt);
            if running {
                any_running = true;
            }
            running
        });
        any_running
    }

    /// Whether any animations are currently running.
    pub fn is_animating(&self) -> bool {
        !self.animations.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tween_linear() {
        let mut tween = Tween::new(0.0, 100.0, Duration::from_secs(1));
        tween.tick(Duration::from_millis(500));
        assert!((tween.value() - 50.0).abs() < 0.1);
        tween.tick(Duration::from_millis(500));
        assert!((tween.value() - 100.0).abs() < 0.1);
    }

    #[test]
    fn tween_ease_in_out() {
        let mut tween = Tween::new(0.0, 1.0, Duration::from_secs(1)).easing(Easing::EaseInOut);
        tween.tick(Duration::from_millis(500));
        // At midpoint, ease-in-out should be near 0.5
        assert!((tween.value() - 0.5).abs() < 0.1);
    }

    #[test]
    fn spring_converges() {
        let mut spring = Spring::new(0.0, 1.0).stiffness(170.0).damping(26.0);
        for _ in 0..200 {
            spring.tick(Duration::from_millis(16));
        }
        assert!((spring.value() - 1.0).abs() < 0.05);
        assert!(spring.is_finished());
    }

    #[test]
    fn easing_boundaries() {
        for easing in [
            Easing::Linear,
            Easing::EaseIn,
            Easing::EaseOut,
            Easing::EaseInOut,
            Easing::EaseOutBounce,
        ] {
            assert!((easing.apply(0.0)).abs() < 0.01, "{easing:?} at 0");
            assert!((easing.apply(1.0) - 1.0).abs() < 0.01, "{easing:?} at 1");
        }
    }
}
