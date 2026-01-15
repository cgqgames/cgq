# CGQ - Card Game Quiz Framework

A Bevy-based game engine for quiz games with card effects, built for the Palestinian History Quiz.

## Architecture

**Built with Bevy ECS:**
- **Components**: Question, DeployedCard, card effect components (EliminateWrongAnswers, ModifyTime, etc.)
- **Resources**: QuizState, GameTimer, Score, CardManager
- **Systems**: quiz_system, card_effect_system, timer_system, input_system

**Card Effects as Bevy Systems:**
Cards modify game state through Bevy's ECS query system. When a card is deployed, it spawns entities with effect components that are processed by specialized systems.

## Running the Game

```bash
# Build and run
cargo run

# Run in release mode for better performance
cargo run --release

# Check for errors without running
cargo check
```

## Project Structure

```
cgq/
├── src/
│   ├── main.rs           # Bevy app setup
│   ├── components.rs     # ECS components (Question, Card effects)
│   ├── resources.rs      # Global state (QuizState, Timer, Score)
│   ├── systems.rs        # Game logic systems
│   └── cards.rs          # YAML card/question loading
├── content/
│   └── palestinian-quiz/
│       ├── cards/        # Card definitions (YAML)
│       │   ├── resistance.yml
│       │   ├── palestinian.yml
│       │   └── negative.yml
│       └── questions/    # Quiz questions (YAML)
│           └── test.yml
└── doc/                  # Planning and technical specs
```

## Card System

Cards are defined in YAML and loaded at runtime. Each card effect becomes a Bevy Component:

**Example: Yaffa Drone Strike Card**
```yaml
id: yaffa_drone_strike
name: "Yaffa Drone Strike"
effects:
  - type: eliminate_wrong_answer
    count: 1
```

When deployed, this spawns an `EliminateWrongAnswers` component that the `card_effect_system` processes, modifying the question's options.

## Controls

- **ENTER**: Start quiz
- **A/B/C/D**: Answer questions
- **N**: Next question (after answering)
- **SPACE**: Pause/Resume game

## Development

Current status:
- ✅ Bevy ECS foundation
- ✅ Component definitions
- ✅ Card YAML loading
- ✅ Quiz game loop (fully playable!)
- ✅ UI rendering (questions, answers, score, timer)
- ✅ Keyboard input handling
- ✅ Question progression
- 🚧 Card deployment system
- 🚧 Card effects on gameplay
- 🚧 Twitch integration
- 🚧 Database persistence
- 🚧 Campaign mode

## Documentation

See `doc/` for detailed planning:
- `planning.md` - High-level design
- `technical-spec.md` - Implementation details
- `card-reference.md` - Card catalog

## Building for Web (WASM)

```bash
# Install wasm target
rustup target add wasm32-unknown-unknown

# Build for web
cargo build --release --target wasm32-unknown-unknown

# Run local server
# (Additional wasm-bindgen setup required)
```
