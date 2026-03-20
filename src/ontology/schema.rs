use serde::{Deserialize, Serialize};

/// Describes the schema of a widget type for agent discoverability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetSchema {
    /// Unique type name (e.g., "Paragraph", "List", "Input").
    pub name: String,
    /// Human-readable description of the widget's purpose.
    pub description: String,
    /// The semantic role this widget type typically plays.
    pub default_role: SemanticRole,
    /// Properties that can be configured on this widget.
    pub properties: Vec<PropertySchema>,
    /// Brief usage example for agents.
    pub usage_hint: Option<String>,
    /// Tags for fuzzy search (e.g., ["text", "display", "paragraph"]).
    pub tags: Vec<String>,
}

/// Describes a single configurable property on a widget.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PropertySchema {
    /// Property name (e.g., "text", "style", "wrap").
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// The type of this property.
    pub property_type: PropertyType,
    /// Whether this property is required.
    pub required: bool,
    /// Default value as JSON.
    pub default_value: Option<serde_json::Value>,
    /// Optional constraints on the property value.
    pub constraints: Vec<PropertyConstraint>,
}

/// Type of a widget property.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PropertyType {
    String,
    Integer,
    Float,
    Boolean,
    Color,
    Style,
    Rect,
    Enum(Vec<String>),
    Array(Box<PropertyType>),
    Object(Vec<PropertySchema>),
    Widget,
    Any,
}

/// A constraint on a property value.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PropertyConstraint {
    /// Minimum numeric value.
    Min(f64),
    /// Maximum numeric value.
    Max(f64),
    /// Minimum string length.
    MinLength(usize),
    /// Maximum string length.
    MaxLength(usize),
    /// Regex pattern the string must match.
    Pattern(String),
    /// Set of allowed values.
    OneOf(Vec<serde_json::Value>),
}

/// The semantic role a widget plays in the UI.
///
/// Agents use roles to understand *what* a widget does without needing to
/// know its concrete type. This enables role-based discovery and interaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SemanticRole {
    /// Displays text or data to the user (read-only).
    Display,
    /// Accepts user input (text fields, checkboxes, etc.).
    Input,
    /// Provides navigation between views or sections.
    Navigation,
    /// Contains and arranges other widgets.
    Container,
    /// Indicates progress or loading state.
    Progress,
    /// Displays a selection from a set of options.
    Selection,
    /// Provides interactive data visualization.
    DataVisualization,
    /// A decorative or structural element (borders, spacers).
    Decoration,
    /// An interactive button or action trigger.
    Action,
    /// A scrollable region.
    Scrollable,
    /// A modal dialog or overlay.
    Modal,
    /// A menu or dropdown.
    Menu,
    /// A toolbar or action bar.
    Toolbar,
    /// A status bar or info display.
    StatusBar,
    /// Custom role with a descriptive string tag.
    Custom(u16),
}

impl std::fmt::Display for SemanticRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Display => write!(f, "display"),
            Self::Input => write!(f, "input"),
            Self::Navigation => write!(f, "navigation"),
            Self::Container => write!(f, "container"),
            Self::Progress => write!(f, "progress"),
            Self::Selection => write!(f, "selection"),
            Self::DataVisualization => write!(f, "data-visualization"),
            Self::Decoration => write!(f, "decoration"),
            Self::Action => write!(f, "action"),
            Self::Scrollable => write!(f, "scrollable"),
            Self::Modal => write!(f, "modal"),
            Self::Menu => write!(f, "menu"),
            Self::Toolbar => write!(f, "toolbar"),
            Self::StatusBar => write!(f, "status-bar"),
            Self::Custom(id) => write!(f, "custom-{id}"),
        }
    }
}
