use crate::core::buffer::Buffer;
use crate::core::rect::Rect;
use crate::core::style::Style;
use crate::ontology::{
    AgentAction, AgentCapability, Discoverable, PropertySchema, PropertyType, SemanticRole,
    WidgetSchema,
};
use crate::widget::block::Block;
use crate::widget::Widget;

/// Image rendering protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageProtocol {
    /// Kitty graphics protocol (used by Kitty, WezTerm, Ghostty).
    Kitty,
    /// iTerm2 inline image protocol.
    ITerm2,
    /// Fallback: render a text placeholder.
    Fallback,
}

/// An image widget that renders inline images in supported terminals.
///
/// Uses the Kitty graphics protocol or iTerm2 inline image protocol when
/// available, with a text fallback for unsupported terminals.
///
/// # Terminal Support
///
/// - **Kitty**: Native support via APC sequences
/// - **WezTerm**: Supports Kitty graphics protocol
/// - **Ghostty**: Supports Kitty graphics protocol
/// - **iTerm2**: Uses iTerm2 inline image protocol
/// - **Other**: Shows a placeholder with image dimensions
///
/// # Example
///
/// ```ignore
/// use louie::widget::image::{Image, ImageProtocol};
///
/// let img = Image::new(include_bytes!("logo.png").to_vec(), "image/png")
///     .protocol(ImageProtocol::Kitty)
///     .fallback_text("[logo 64x64]");
/// ```
#[derive(Debug, Clone)]
pub struct Image {
    /// Raw image data (e.g., PNG bytes).
    data: Vec<u8>,
    /// MIME type of the image data.
    mime_type: String,
    /// Rendering protocol to use.
    protocol: ImageProtocol,
    /// Optional block wrapping.
    block: Option<Block>,
    /// Style for fallback text.
    style: Style,
    /// Fallback text shown when the protocol is Fallback or unsupported.
    fallback_text: Option<String>,
    /// Maximum width in terminal columns.
    max_width: Option<u16>,
    /// Maximum height in terminal rows.
    max_height: Option<u16>,
}

impl Image {
    /// Create a new image widget from raw bytes and MIME type.
    pub fn new(data: Vec<u8>, mime_type: impl Into<String>) -> Self {
        Self {
            data,
            mime_type: mime_type.into(),
            protocol: ImageProtocol::Fallback,
            block: None,
            style: Style::default(),
            fallback_text: None,
            max_width: None,
            max_height: None,
        }
    }

    /// Set the rendering protocol.
    pub fn protocol(mut self, protocol: ImageProtocol) -> Self {
        self.protocol = protocol;
        self
    }

    pub fn block(mut self, block: Block) -> Self {
        self.block = Some(block);
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set fallback text for unsupported terminals.
    pub fn fallback_text(mut self, text: impl Into<String>) -> Self {
        self.fallback_text = Some(text.into());
        self
    }

    pub fn max_width(mut self, w: u16) -> Self {
        self.max_width = Some(w);
        self
    }

    pub fn max_height(mut self, h: u16) -> Self {
        self.max_height = Some(h);
        self
    }

    /// Detect the best protocol based on environment variables.
    pub fn detect_protocol() -> ImageProtocol {
        // Check for Kitty
        if std::env::var("KITTY_WINDOW_ID").is_ok() {
            return ImageProtocol::Kitty;
        }

        // Check TERM_PROGRAM for known terminals
        if let Ok(term) = std::env::var("TERM_PROGRAM") {
            match term.as_str() {
                "WezTerm" | "ghostty" => return ImageProtocol::Kitty,
                "iTerm.app" => return ImageProtocol::ITerm2,
                _ => {}
            }
        }

        ImageProtocol::Fallback
    }

    /// Encode image data as base64.
    fn base64_data(&self) -> String {
        use std::fmt::Write;
        let bytes = &self.data;
        const CHARS: &[u8; 64] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

        let mut result = String::with_capacity(bytes.len().div_ceil(3) * 4);
        for chunk in bytes.chunks(3) {
            let b0 = chunk[0] as u32;
            let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
            let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
            let triple = (b0 << 16) | (b1 << 8) | b2;

            let _ = result.write_char(CHARS[((triple >> 18) & 0x3F) as usize] as char);
            let _ = result.write_char(CHARS[((triple >> 12) & 0x3F) as usize] as char);
            if chunk.len() > 1 {
                let _ = result.write_char(CHARS[((triple >> 6) & 0x3F) as usize] as char);
            } else {
                result.push('=');
            }
            if chunk.len() > 2 {
                let _ = result.write_char(CHARS[(triple & 0x3F) as usize] as char);
            } else {
                result.push('=');
            }
        }
        result
    }

    /// Generate a Kitty graphics protocol escape sequence.
    ///
    /// This returns the escape sequence string that the backend should write
    /// to the terminal. The widget stores it in the buffer as a special cell.
    fn kitty_sequence(&self, cols: u16, rows: u16) -> String {
        let b64 = self.base64_data();
        // Kitty: transmit image with direct data
        // a=T (transmit + display), f=100 (PNG), t=d (direct data)
        // c=cols, r=rows for display size
        format!("\x1b_Ga=T,f=100,t=d,c={},r={};{}\x1b\\", cols, rows, b64)
    }

    /// Generate an iTerm2 inline image escape sequence.
    fn iterm2_sequence(&self, cols: u16, rows: u16) -> String {
        let b64 = self.base64_data();
        format!(
            "\x1b]1337;File=inline=1;width={};height={};preserveAspectRatio=1:{}\x07",
            cols, rows, b64
        )
    }
}

impl Widget for Image {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let inner = if let Some(ref block) = self.block {
            let inner = block.inner(area);
            block.clone().render(area, buf);
            inner
        } else {
            area
        };

        if inner.is_empty() {
            return;
        }

        let display_w = self.max_width.unwrap_or(inner.width).min(inner.width);
        let display_h = self.max_height.unwrap_or(inner.height).min(inner.height);

        match self.protocol {
            ImageProtocol::Kitty => {
                // Store the Kitty escape in the top-left cell of the image area.
                // The terminal backend will write it directly.
                let seq = self.kitty_sequence(display_w, display_h);
                buf[(inner.x, inner.y)].set_symbol(&seq);
            }
            ImageProtocol::ITerm2 => {
                let seq = self.iterm2_sequence(display_w, display_h);
                buf[(inner.x, inner.y)].set_symbol(&seq);
            }
            ImageProtocol::Fallback => {
                let text = self.fallback_text.as_deref().unwrap_or({
                    // Can't return a reference to a temporary, so use a static
                    "[image]"
                });
                let display = if text.len() as u16 > inner.width {
                    &text[..inner.width as usize]
                } else {
                    text
                };
                let x = inner.x + (inner.width.saturating_sub(display.len() as u16)) / 2;
                let y = inner.y + inner.height / 2;
                buf.set_string(x, y, display, self.style);
            }
        }
    }
}

impl Discoverable for Image {
    fn schema() -> WidgetSchema {
        WidgetSchema {
            name: "Image".into(),
            description: "An inline image widget using Kitty/iTerm2 graphics protocols.".into(),
            default_role: SemanticRole::Display,
            properties: vec![
                PropertySchema {
                    name: "data".into(),
                    description: "Raw image bytes (e.g., PNG).".into(),
                    property_type: PropertyType::String, // base64 encoded when sent via JSON
                    required: true,
                    default_value: None,
                    constraints: vec![],
                },
                PropertySchema {
                    name: "mime_type".into(),
                    description: "MIME type of the image data (e.g., image/png).".into(),
                    property_type: PropertyType::String,
                    required: true,
                    default_value: None,
                    constraints: vec![],
                },
                PropertySchema {
                    name: "protocol".into(),
                    description: "Image protocol: Kitty, ITerm2, or Fallback.".into(),
                    property_type: PropertyType::Enum(vec![
                        "Kitty".into(),
                        "ITerm2".into(),
                        "Fallback".into(),
                    ]),
                    required: false,
                    default_value: Some(serde_json::json!("Fallback")),
                    constraints: vec![],
                },
            ],
            actions: vec![],

            usage_hint: Some(
                "Use Image::detect_protocol() to auto-select the best rendering method.".into(),
            ),
            tags: vec![
                "image".into(),
                "graphics".into(),
                "visual".into(),
                "media".into(),
            ],
        }
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Display
    }

    fn agent_state(&self) -> serde_json::Value {
        serde_json::json!({
            "mime_type": self.mime_type,
            "data_size": self.data.len(),
            "protocol": format!("{:?}", self.protocol),
        })
    }

    fn execute_action(
        &mut self,
        _action: &str,
        _params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        Err("Image has no executable actions".into())
    }
}
