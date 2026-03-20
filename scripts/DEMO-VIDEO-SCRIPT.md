# Demo Video Script — "Why AI Agents Need a UI Ontology"

**Duration**: 90 seconds  
**Format**: Terminal recording (asciinema or Screen Studio)  
**Resolution**: 1920×1080, dark terminal theme  

---

## Recording Option A: Automated (Recommended)

```sh
# Build the server
cargo build --release --bin louie-server

# Record with asciinema
asciinema rec demo.cast -c "python3 scripts/louie-demo.py"

# Convert to GIF or SVG for embedding
# Option 1: agg (asciinema GIF generator)
agg demo.cast demo.gif --cols 100 --rows 35

# Option 2: svg-term for SVG
svg-term --in demo.cast --out demo.svg --window
```

The `scripts/louie-demo.py` script runs the full demo automatically with dramatic timing and color-coded output. It walks through:

1. Spawning the server
2. Pinging for connectivity
3. Discovering all widget types via `query_ontology`
4. Inspecting a Gauge widget's full schema
5. Reading the UI tree
6. Injecting 3 key presses to control the counter
7. Verifying the state changed
8. Clean shutdown

---

## Recording Option B: Manual Shot List

For a more polished video with voiceover or captions:

### Shot 1: The Problem (0:00–0:15)

**On screen**: Split terminal. Left side: raw terminal output of a TUI app (garbled ANSI escapes). Right side: an LLM prompt trying to parse it.

**Caption/voiceover**: 
> "AI agents are blind to the UIs they drive. They parse ANSI escape sequences, guess at widget boundaries, and break when layouts change."

### Shot 2: The Alternative (0:15–0:25)

**On screen**: `echo '{"type":"ping"}' | louie-server` — clean JSON response.

**Caption**: 
> "What if the UI could describe itself?"

### Shot 3: Discovery (0:25–0:40)

**On screen**: `query_ontology` response scrolling through widget schemas — Input, Gauge, List, Editor, SelectList, each with properties, types, constraints.

**Caption**: 
> "Every widget exposes its type, properties, constraints, and available actions through a typed ontology. The agent asks 'what exists?' and gets structured answers."

### Shot 4: Inspection (0:40–0:50)

**On screen**: `get_tree` response showing the UI node tree with agent IDs, semantic roles, capabilities.

**Caption**: 
> "The full UI tree is available as structured JSON — positions, state, capabilities. No screen-scraping needed."

### Shot 5: Control (0:50–1:05)

**On screen**: Three `inject_event` commands with `{"kind":"key","code":"Up"}`, each followed by a `get_state` showing the counter incrementing.

**Caption**: 
> "The agent controls the app through the same protocol — execute named actions, inject keyboard events, observe the result."

### Shot 6: The Pitch (1:05–1:20)

**On screen**: Side-by-side comparison table:

```
              Screen Scraping    Louie Protocol
Discovery     None               Full ontology
Reliability   Fragile            Robust
Maintenance   Break on change    Self-describing
```

**Caption**: 
> "A self-describing UI ontology. For terminals. In Rust."

### Shot 7: Call to Action (1:20–1:30)

**On screen**: `github.com/nervosys/louie`

**Caption**: 
> "Open source under AGPL. Commercial licenses available."

---

## Distribution

- **GitHub README**: Embed the GIF or link the asciinema recording
- **Twitter/X**: Upload as MP4 (convert via `ffmpeg -i demo.cast.gif -pix_fmt yuv420p demo.mp4`)
- **Blog post**: Embed the SVG or asciinema player
- **Slide deck**: First 3 shots as static screenshots, link to full recording

---

## Recording Tips

- Use a **large font** (18-20pt) for readability on social media
- Use a **dark theme** with high contrast (Catppuccin Mocha, Tokyo Night, etc.)
- Keep terminal width at **100 columns** max — wider is harder to read in embeds
- The Python demo script has built-in dramatic timing — adjust `delay` params if needed
- For voiceover, record audio separately and sync in post
