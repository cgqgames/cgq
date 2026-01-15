# Sample Quiz

This is a minimal example showing how to structure quiz content for the CGQ framework.

## Running This Example

From the project root:

```bash
cargo run -- --quiz examples/sample-quiz/questions.yml
```

Or with cards:

```bash
cargo run -- --quiz examples/sample-quiz/questions.yml --cards examples/sample-quiz/cards.yml
```

## File Structure

```
sample-quiz/
├── questions.yml    # Quiz questions
├── cards.yml        # Card definitions (optional)
└── README.md        # This file
```

## Question Format

See `questions.yml` for the structure. Each question needs:
- `id`: Unique identifier
- `text`: The question text
- `options`: Array of 4 options (a, b, c, d) with one marked `correct: true`
- `points`: Points awarded for correct answer
- `explanation`: (Optional) Shown after answer
- `source`: (Optional) Citation or reference
- `tags`: (Optional) Categories for organization

## Card Format

See `cards.yml` for the structure. Each card needs:
- `id`: Unique identifier
- `name`: Display name
- `type`: Card category (resistance, palestinian, politics, negative)
- `effects`: Array of effects that modify gameplay
- `visual`: (Optional) Image and sound files

## Creating Your Own Quiz

1. Copy this directory structure
2. Edit `questions.yml` with your questions
3. (Optional) Create custom cards in `cards.yml`
4. Run with `cargo run -- --quiz path/to/your/questions.yml`

The framework handles the rest!
