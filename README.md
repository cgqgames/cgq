# CGQ — Card Game Quiz

A data-driven card-based quiz game engine built with Rust and Bevy. Content
creators define questions and cards in data files; the engine handles timing,
scoring, card effects, and optional Twitch chat integration.

## Build & Run

```
cargo run -- --quiz examples/sample-quiz/questions.yml
```

### CLI flags

| Flag | Default | Description |
|------|---------|-------------|
| `--quiz`, `-q` | `examples/sample-quiz/questions.yml` | Path to questions YAML |
| `--cards`, `-c` | `<quiz dir>/cards/` | Directory of card TOML files |
| `--ui-config`, `-u` | built-in defaults | Path to UI config TOML |
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

```
content/<quiz-name>/
  questions.yml          # question set (YAML)
  cards/
    <card-id>.toml       # one card per file (TOML)
```

See `examples/sample-quiz/` for a working reference and `doc/planning.md` for
the full design document.

## WASM

```
./build-wasm.sh
```

Produces a web build in `web/`. Serve with any static file server.
