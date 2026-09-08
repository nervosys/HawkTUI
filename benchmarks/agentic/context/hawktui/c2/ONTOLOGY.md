# Hawk TUI widget ontology (generated)

21 widget types. Format: `property: type` per line.

## BarChart — A bar chart for visualizing categorical data with grouped bars. (data-visualization)
- groups*: [object]
- bar_width: int
- direction: enum(Vertical|Horizontal)

```rust
Use for categorical comparisons. Each group can have multiple bars for multi-series data.
```

## Block — A container widget with borders, titles, and background styling. (container)
- title: string
- borders: enum(none|all|top|bottom|left|right)
- border_type: enum(plain|rounded|double|thick)

```rust
Block::bordered().title("My Panel")
```

## Calendar — A month-view calendar grid with day-of-week headers and day highlighting. (display)
- year*: int
- month*: int

```rust
Calendar::new(2026, 3).show_header(true)
```

## CancellableLoader — An animated spinner/loader with cancellation support. (status-bar)
- message: string
- cancelled: bool

```rust
CancellableLoader::new("Processing...").tick(n)
```

## Canvas — A freeform drawing surface using braille characters for sub-cell resolution. (data-visualization)
- x_bounds: [float]
- y_bounds: [float]

```rust
Canvas::new().x_bounds([0.0, 100.0]).line(CanvasLine { x1: 0.0, y1: 0.0, x2: 100.0, y2: 50.0, color: Color::White })
```

## Chart — An XY chart for line and scatter plots with multiple datasets. (data-visualization)
- datasets*: [object]
- x_axis*: object
- y_axis*: object

```rust
Use for plotting numerical data trends. Braille markers give sub-cell resolution.
```

## Editor — A multi-line text editor with cursor movement, line editing, and scrolling. (input)
- show_line_numbers: bool

```rust
Editor::new().show_line_numbers(true)
```

## Gauge — A horizontal progress bar displaying a ratio or percentage. (progress)
- ratio*: float
- label: string

```rust
Gauge::new().percent(42).label("Loading...")
```

## Image — An inline image widget using Kitty/iTerm2 graphics protocols. (display)
- data*: string
- mime_type*: string
- protocol: enum(Kitty|ITerm2|Sixel|HalfBlock|Fallback)

```rust
Use Image::detect_protocol() to auto-select the best rendering method.
```

## Input — A single-line text input field with cursor navigation and editing. (input)
- placeholder: string

```rust
Input::new().placeholder("Type here...")
```

## LineGauge — A thin, single-line progress bar using line-drawing characters. (progress)
- ratio*: float
- label: string

```rust
LineGauge::new().percent(65).label("Progress")
```

## List — A scrollable list with item selection, highlight, and keyboard navigation. (selection)
- items*: [string]
- highlight_symbol: string

```rust
List::new(["Item 1", "Item 2"]).highlight_symbol(">> ")
```

## Loader — An animated spinner/loader with an optional message. (status-bar)
- message: string
- spinner_style: string

```rust
Loader::new("Loading...").spinner_style(SpinnerStyle::Braille).tick(n)
```

## Markdown — Renders Markdown text with headings, bold, italic, code, lists, and blockquotes. (display)
- source*: string

```rust
Markdown::new("# Hello\n\nSome **bold** text")
```

## Paragraph — Displays styled text with optional wrapping, scrolling, and alignment. (display)
- text*: string
- wrap: enum(none|word|char)
- alignment: enum(left|center|right)
- scroll: object

```rust
Paragraph::new("Hello, world!").centered()
```

## Scrollbar — A scroll indicator showing viewport position within content. (scrollable)
- orientation: enum(Vertical|Horizontal)

```rust
Scrollbar::new(ScrollbarOrientation::Vertical)
```

## SelectList — An interactive select list supporting single or multi-select with filtering. (input)
- mode: string
- items*: [string]

```rust
SelectList::new(items).mode(SelectMode::Multi)
```

## SettingsList — An interactive settings list with cycleable key-value pairs. (navigation)
- settings*: [object]

```rust
Navigate with up/down, cycle values with Enter or left/right.
```

## Sparkline — A compact chart displaying data trends as vertical bars. (data-visualization)
- data*: [int]
- max: int

```rust
Sparkline::new(vec![0, 1, 3, 7, 5, 2])
```

## Table — A data table with column headers, row selection, and scrolling. (data-visualization)
- columns*: [object]
- rows*: [[string]]

```rust
Table::new([TableColumn::new("Name", TableColumnWidth::Fill)], [TableRow::new(["Alice"])])
```

## Tabs — A horizontal tab bar for section navigation. (navigation)
- titles*: [string]
- selected: int

```rust
Tabs::new(["Tab 1", "Tab 2"]).select(0)
```

