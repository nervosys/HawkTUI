//! Core primitives for terminal rendering.
//!
//! This module provides the foundational types that all other louie modules build on:
//!
//! - [`buffer::Buffer`] — 2D grid of styled cells (the render target)
//! - [`cell::Cell`] — A single terminal cell with character, style, and width
//! - [`rect::Rect`] — Rectangular area with position and size, used everywhere for layout
//! - [`style::Style`] — Foreground/background colors and text modifiers (bold, italic, etc.)
//! - [`text::Text`] — Rich text composed of [`text::Line`]s of styled [`text::Span`]s

pub mod buffer;
pub mod cell;
pub mod rect;
pub mod reflow;
pub mod style;
pub mod text;
