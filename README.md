# CGQ — Card Game Quiz

A data-driven card-based quiz game engine built with Rust and Bevy. Content
creators define questions and cards in data files; the engine handles timing,
scoring, card effects, and optional Twitch chat integration.

## Build & Run

```
cargo run -- --config-dir examples/sample-quiz/etc
```

### CLI flags

| Flag | Default | Description |
|------|---------|-------------|
| `--config-dir`, `-C` | `examples/sample-quiz/etc` | Root of the quiz configuration tree |
| `--twitch-channel`, `-t` | disabled | Twitch channel for chat integration |
| `--chat-threshold` | 3 | Votes needed for chat consensus |
| `--live` | off | Chroma-key green background for OBS |

## Controls

| Key | Action |
|-----|--------|
| Enter | Start game |
| Space | Pause / resume |
| A / B / C / D | Submit answer |
| 1 / 2 / 3 / 4 | Deploy card from board slot |

## Content layout

A quiz is a directory. The engine recursively walks it and deep-merges every
`*.toml` and `*.json` file into a single configuration tree. File layout is
purely organizational — the merged tree is what the engine sees.

```
<config-dir>/
  game.toml       # [game] title, passing_grade, timer_seconds
  ui.toml         # [ui.*] layout, colors, fonts
  questions.toml  # [questions.<id>] one entry per question
  cards/
    fifty_fifty.toml   # [cards.fifty_fifty] ...
    time_bonus.toml    # [cards.time_bonus] ...
```

Cards, questions, game settings and UI config can be organized however you
want. One card per file, all cards in one file, mixed JSON and TOML — all
equivalent. Merge rules: tables merge recursively; arrays and scalars replace.

See `examples/sample-quiz/etc/` for a working reference and `doc/planning.md`
for the full design document.

## WASM

```
./build-wasm.sh
```

Produces a web build in `web/`. Serve with any static file server.
