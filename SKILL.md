# ratatui 0.30 — Reference & Pitfall Guide

You are helping write code for **ratatui 0.30**. This version introduced breaking changes and new
APIs that are not yet well represented in AI training data. Prioritize the patterns in this file
over anything you learned during training. When in doubt, prefer the explicitly correct examples
shown here.

MSRV: **1.86.0**, Rust **2024 edition**.

---

## Imports

```rust
use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    layout::{Alignment, Constraint, Flex, Layout},
    prelude::Stylize,                          // shorthand style methods (.bold(), .yellow(), etc.)
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
};
```

crossterm is re-exported from ratatui — do not add it as a separate dependency.

---

## Minimal working example

```rust
fn main() -> std::io::Result<()> {
    ratatui::run(|terminal| loop {
        terminal.draw(|frame| {
            let [left, right] = Layout::horizontal([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .areas(frame.area());

            frame.render_widget(Block::bordered().title("Left"), left);
            frame.render_widget(Block::bordered().title("Right"), right);
        })?;

        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Char('q') {
                break Ok(());
            }
        }
    })
}
```

---

## API Reference

### `ratatui::run()`

Added in 0.30. Handles `init()`/`restore()` and panic hooks around the closure. Returns
`std::io::Result<()>`. The event loop is yours to manage inside. `terminal` does not need `mut`.

Accepts a closure or a plain function taking `&mut DefaultTerminal`:

```rust
// closure form
fn main() -> std::io::Result<()> {
    ratatui::run(|terminal| loop {
        terminal.draw(|frame| { ... })?;
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break Ok(()),
                _ => {}
            }
        }
    })
}

// function form — run() passes &mut DefaultTerminal
fn run(terminal: &mut DefaultTerminal) -> std::io::Result<()> { ... }
ratatui::run(run)?;
```

### Layout

```rust
// vertical split
let [top, bottom] = Layout::vertical([
    Constraint::Length(3),   // fixed height in cells
    Constraint::Fill(1),     // takes remaining space
]).areas(frame.area());

// horizontal split of an existing rect
let [left, right] = Layout::horizontal([
    Constraint::Percentage(50),
    Constraint::Percentage(50),
]).areas(bottom);

// alternative: Rect::layout() (new in 0.30, equivalent to layout.areas())
let layout = Layout::vertical([Constraint::Fill(1); 2]);
let [top, main] = frame.area().layout(&layout);
```

Constraint types: `Length(n)` fixed, `Fill(n)` weighted remainder, `Percentage(n)`,
`Min(n)`, `Max(n)`, `Ratio(a, b)`.

Flex options (aligns with CSS flexbox):
- `Flex::Start` — pack to start
- `Flex::End` — pack to end
- `Flex::Center` — center with equal space on both sides
- `Flex::SpaceBetween` — even space between items, none at edges
- `Flex::SpaceAround` — space around items; middle gaps are twice the edge gaps
- `Flex::SpaceEvenly` — equal space between all items and edges
- `Flex::Legacy` — excess space at end (old default)
- `Flex::Stretch` — stretch items to fill (default)

Flex only has visible effect when constraints leave remaining space (i.e. with `Length`,
`Min`/`Max` — not when everything is consumed by `Fill` or `Percentage(100)`).

### Centering

```rust
// center in both axes
let area = frame.area().centered(
    Constraint::Percentage(60),
    Constraint::Ratio(1, 3),
);

// center on one axis only
let area = frame.area().centered_horizontally(Constraint::Length(40));
let area = frame.area().centered_vertically(Constraint::Percentage(50));
```

`centered_rect()` helper functions seen in older tutorials and AI output are obsolete —
use `Rect::centered()` / `Rect::centered_horizontally()` / `Rect::centered_vertically()`.

### Block

```rust
Block::bordered()                              // shorthand for .borders(Borders::ALL)
    .title(Line::from("Top Left"))             // default: top-left
    .title(Line::from("Center").centered())
    .title(Line::from("Right").right_aligned())
    .title_bottom(Line::from("Bottom").centered())
    .border_type(BorderType::Rounded)          // Plain (default), Thick, Double, Rounded,
                                               // LightDoubleDashed, HeavyDoubleDashed,
                                               // LightTripleDashed, HeavyTripleDashed,
                                               // LightQuadrupleDashed, HeavyQuadrupleDashed
    .border_style(Style::default().fg(Color::Yellow))  // color of border characters only
    .style(Style::default().bg(Color::DarkGray))       // inner area background
    .borders(Borders::TOP | Borders::BOTTOM)   // specific sides (bitflag)
```

`border_style` applies to border characters only. `.style()` applies to the inner area.

`block.inner(rect)` returns the `Rect` inside the border — useful when manually rendering
multiple things inside one bordered region:

```rust
let block = Block::bordered().title("Panel");
let inner = block.inner(some_rect);
frame.render_widget(&block, some_rect);
frame.render_widget("content", inner);
```

### Paragraph

```rust
Paragraph::new("plain text")
    .block(Block::bordered().title("Title"))
    .alignment(Alignment::Center)
    .wrap(Wrap { trim: true })
    .scroll((row, col))
    .style(Style::default().fg(Color::White))
```

For mixed-style text use `Line`/`Span`:

```rust
Paragraph::new(Line::from_iter([
    Span::from("normal "),
    Span::styled("yellow", Style::default().fg(Color::Yellow)),
    "bold".bold().into(),
]))
```

### Line and Span

`Span` is the atomic unit — a string with a single style. `Line` is a row of `Span`s.
`Text` is multiple `Line`s (what `Paragraph` actually takes internally).

```rust
Line::from("plain")
Line::from("centered").centered()
Line::from("right").right_aligned()
Line::from("styled").style(Style::default().fg(Color::Red))
```

`&str` implements `Into<Span>`, so string literals can be converted with `.into()` or via
`Span::from()`. For multi-span lines, use `Line::from_iter` with an array — no heap allocation:

```rust
// idiomatic — array, no allocation
Line::from_iter([
    Span::from("normal "),
    "styled".yellow().into(),
    "bold".blue().bold().into(),
])

// also works via into_iter + collect
["normal ".into(), "styled".yellow().into()]
    .into_iter()
    .collect::<Line>()

// only use vec! when building spans dynamically (e.g. in a loop)
let mut spans: Vec<Span> = vec![];
for item in items { spans.push(item.into()); }
Line::from(spans)
```

With the optional `ratatui-macros` crate (officially maintained, separate dependency):

```rust
use ratatui_macros::line;
line!["normal ", "styled".yellow(), "bold".blue().bold()]
```

### Style

`Style::new()` and `Style::default()` are identical. `Style::new()` can be used in `const` context:

```rust
const MY_STYLE: Style = Style::new().blue().on_black().bold();
```

`Stylize` shorthand works on `&str`, `Span`, `Line`, and most widgets:

```rust
"text".yellow().bold()
Block::bordered().red()     // fg red
Block::bordered().on_blue() // bg blue
```

Color constructors:

```rust
Color::Red
Color::Rgb(255, 128, 0)
Color::Indexed(196)
Color::from([255, 0, 0])      // from array (new in 0.30)
Color::from((255, 0, 0))      // from tuple (new in 0.30)
```

### Alignment

`Alignment` was renamed to `HorizontalAlignment` in 0.30. The old name still works as an alias:

```rust
use ratatui::layout::HorizontalAlignment;  // new preferred name
use ratatui::layout::Alignment;            // still works
```

### List (stateful)

```rust
let list = List::new(["Item 1", "Item 2", "Item 3"])
    .block(Block::bordered().title("List"))
    .highlight_style(Style::default().fg(Color::Yellow).bold())
    .highlight_symbol(Line::from(">> ").red())  // accepts Into<Line> in 0.30
    .scroll_padding(1);

let mut list_state = ListState::default().with_selected(Some(0));

// must use render_stateful_widget
frame.render_stateful_widget(&list, area, &mut list_state);

// update selection
list_state.select_next();
list_state.select_previous();
```

Without `highlight_style`, selection is invisible even though `ListState` tracks it.

### Rendering widgets by reference

Widgets implement `Widget for &W` in 0.30. Always render by reference so widgets declared
outside the draw closure are reused each frame without being consumed:

```rust
// correct — declared once outside the loop, borrowed each frame
frame.render_widget(&my_widget, area);
frame.render_stateful_widget(&my_widget, area, &mut state);

// works but consumes the widget — forces redeclaring it inside draw every frame
frame.render_widget(my_widget, area);
```

### Resize handling

Resize events unblock `event::read()` automatically. The next `terminal.draw()` picks up the
new dimensions via `frame.area()`. No poll loop needed for resize:

```rust
match event::read()? {
    Event::Key(key) => { ... }
    Event::Resize(_, _) => {}  // loop continues, next draw picks up new size
    _ => {}
}
```

### `event::read()` vs `event::poll()`

`event::read()` blocks until an event arrives. Correct for purely reactive apps.

`event::poll(duration)` is non-blocking — use it when the app must update without user input
(timers, animations, background tasks):

```rust
if event::poll(Duration::from_millis(100))? {
    if let Event::Key(key) = event::read()? { ... }
}
// falls through here every 100ms regardless of input
```

---

## Known Pitfalls

### `ratatui::run()` does not loop

The closure runs once. Without an explicit `loop {}` inside, the app draws one frame and exits.

```rust
// wrong — exits immediately after one frame
ratatui::run(|terminal| {
    terminal.draw(|frame| { ... })?;
    Ok(())
})

// correct
ratatui::run(|terminal| loop {
    terminal.draw(|frame| { ... })?;
    ...
    break Ok(());
})
```

### `ratatui::run()` closure receives `&mut DefaultTerminal`, not a `Frame`

```rust
// wrong — terminal is &mut DefaultTerminal, not Frame
ratatui::run(|frame| {
    frame.render_widget(...);
})

// correct — Frame comes from terminal.draw()
ratatui::run(|terminal| loop {
    terminal.draw(|frame| {
        frame.render_widget(...);
    })?;
    ...
})
```

### `Flex::SpaceAround` behavior changed in 0.30

The old `Flex::SpaceAround` is now `Flex::SpaceEvenly`. The new `Flex::SpaceAround` distributes
space differently (middle gaps are twice the edge gaps). Code using `SpaceAround` from pre-0.30
tutorials expecting even distribution should use `SpaceEvenly` instead.

```rust
// pre-0.30 SpaceAround behavior (equal space everywhere) is now:
.flex(Flex::SpaceEvenly)

// new SpaceAround distributes space around items with doubled middle spacing
.flex(Flex::SpaceAround)
```

### `block::Title` removed in 0.30

```rust
// wrong — Title does not exist in 0.30
use ratatui::widgets::block::Title;
Block::bordered().title(Title::from("hello").alignment(Alignment::Center));

// correct
Block::bordered()
    .title(Line::from("hello").centered())
    .title_bottom(Line::from("footer"));
```

`widgets::block::Position` is also removed — replaced by `widgets::TitlePosition`.

### `.split()` is legacy

```rust
// legacy — returns Vec<Rect>, no destructuring, no compile-time size check
let chunks = layout.split(frame.area());
let left = chunks[0];

// correct
let [left, right] = layout.areas(frame.area());
```

### `line![]` macro requires a separate crate

`line![]` is from `ratatui-macros`, not ratatui itself. Use `Line::from_iter` without it:

```rust
// requires ratatui-macros in Cargo.toml
line!["hello", "world".yellow()]

// correct without extra dependency — no allocation
Line::from_iter([Span::from("hello"), "world".yellow().into()])
```

### `Line::from` does not accept arrays or tuples directly

```rust
// wrong — From<[Span; N]> and From<(Span, Span)> are not implemented
Line::from([Span::from("a"), Span::from("b")])
Line::from((Span::from("a"), Span::from("b")))

// correct — from_iter accepts any IntoIterator<Item = Span>
Line::from_iter([Span::from("a"), Span::from("b")])

// also correct when you already have a Vec<Span>
Line::from(my_vec_of_spans)
```

### Avoid `vec!` for `Line` construction

`vec!` heap-allocates. For fixed span lists use `from_iter` with an array:

```rust
// avoid
Line::from(vec!["a".into(), "b".yellow().into()])

// prefer
Line::from_iter(["a".into(), "b".yellow().into()])
```

`vec!` is still appropriate when building spans dynamically (loops, conditionals, unknown length).


### Stateful widgets require `render_stateful_widget`

```rust
// wrong — compiles but state is ignored
frame.render_widget(&list, area);

// correct
frame.render_stateful_widget(&list, area, &mut list_state);
```

### `highlight_style` required for visible list selection

`ListState` tracks selection but `List` renders it invisibly without a highlight style:

```rust
// selection exists but is invisible
let list = List::new([...]);

// correct
let list = List::new([...]).highlight_style(Style::default().fg(Color::Yellow).bold());
```

### Flex has no effect without leftover space

```rust
// Flex has no effect — Fill(1) consumes all remaining space
Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)])
    .flex(Flex::SpaceBetween)

// Flex works — Length leaves remaining space to distribute
Layout::horizontal([Constraint::Length(20), Constraint::Length(20)])
    .flex(Flex::SpaceBetween)
```

### crossterm is re-exported — do not add as a separate dependency

```toml
# wrong
[dependencies]
ratatui = "0.30"
crossterm = "0.28"

# correct
[dependencies]
ratatui = "0.30"
```

```rust
// correct import path
use ratatui::crossterm::event::{self, Event, KeyCode};
```

### `centered_rect()` helper is obsolete

```rust
// obsolete helper seen in old tutorials — do not generate this
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect { ... }

// correct
let area = frame.area().centered(Constraint::Percentage(60), Constraint::Percentage(40));
let area = frame.area().centered_horizontally(Constraint::Length(40));
let area = frame.area().centered_vertically(Constraint::Percentage(50));
```
