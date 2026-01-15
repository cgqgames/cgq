# CGQ (Card Game Engine) - Framework Planning Document

## Project Overview

**CGQ is a general-purpose framework for creating interactive card-based games**, not a single application. The goal is to build a flexible engine that content creators can use to build various game experiences by providing game type definitions, rulesets, card definitions, and content as data.

**The framework provides**:
- **Pluggable game type system** - Quiz, Grid-based (Battleship), Deck-builder, etc.
- **Component-based architecture** - Composable game state pieces
- **Card effect interceptor system** - Cards can modify any component query/mutation
- Core game engine with timer, scoring, and state management
- Twitch chat integration for collaborative gameplay
- Rendering system for OBS/browser sources
- Campaign/progression system
- Content loading from configuration files

**The Palestinian history quiz described in the transcript is the first implementation** built using this framework, serving as both a proof-of-concept and a reference implementation.

### Vision: "The Unity of Card-Based Interactive Games"

Just as Unity allows creators to build various game types without starting from scratch, CGQ should enable creators to build interactive card-based experiences by:
1. Choosing a game type (Quiz, Grid, Deck-builder, etc.) or creating custom types
2. Defining game components and their interactions
3. Creating cards with effects that modify game behavior
4. Writing content in JSON/YAML (questions, grid layouts, etc.)
5. Compiling into a playable game

**Current State**: Manual operation requiring constant host intervention
**Target State**: Fully automated framework supporting multiple game types with content-driven generation

---

## Supported Game Types

The framework supports multiple game types through a trait-based plugin system:

### 1. Quiz Game (Initial Implementation)
- **Components**: Questions, Answers, Score, Timer
- **Mechanics**: Answer submission, consensus voting, scoring
- **Cards**: Eliminate answers, modify points, adjust timer
- **Example**: Palestinian history educational quiz

### 2. Grid Game (e.g., Battleship)
- **Components**: Grid, Positions, Ships, Hits/Misses
- **Mechanics**: Position selection, hit detection, ship placement
- **Cards**: Reveal positions, modify grid, extra shots, shields
- **Example**: Naval battle quiz where correct answers reveal grid positions

### 3. Deck Builder
- **Components**: Deck, Hand, Discard pile, Resources
- **Mechanics**: Card drawing, playing, resource management
- **Cards**: Modify deck composition, resource generation
- **Example**: Build a deck by answering questions correctly

### 4. Custom Game Types
Content creators can define new game types by implementing the `GameType` trait

---

## Core Architecture Concepts

### Game Type as a Trait

Each game type implements a common interface but defines its own mechanics:

```rust
trait GameType {
    type State;
    type Input;
    type Output;

    fn initialize(&self, config: &GameConfig) -> Self::State;
    fn process_input(&self, state: &mut Self::State, input: Self::Input);
    fn check_win_condition(&self, state: &Self::State) -> Option<GameResult>;
    fn get_components(&self, state: &Self::State) -> Vec<Component>;
}
```

### Component System

Game state is decomposed into **components** (data structures):

```typescript
// Quiz game components
interface QuestionComponent {
  id: string
  text: string
  options: Option[]
  points: number
}

interface ScoreComponent {
  current: number
  passing_grade: number
}

interface TimerComponent {
  remaining_ms: number
  running: boolean
}

// Grid game components
interface GridComponent {
  width: number
  height: number
  cells: Cell[][]
}

interface PositionComponent {
  x: number
  y: number
  revealed: boolean
  occupied: boolean
}
```

### Card Effect Interceptor System

Cards define **interceptors** that hook into component queries and mutations:

```typescript
interface CardEffect {
  // Which component queries to intercept
  intercepts: ComponentQuery[]

  // Transform function
  transform: (query: ComponentQuery, result: any) => any

  // When to apply (before/after query, on mutation, etc.)
  timing: 'before' | 'after' | 'on_mutation'
}

// Example: "Double Points" card
const doublePointsCard: Card = {
  id: "double_points",
  effects: [
    {
      intercepts: [{ component: 'Score', operation: 'add' }],
      transform: (query, result) => ({
        ...result,
        amount: result.amount * 2
      }),
      timing: 'before'
    }
  ]
}

// Example: Grid reveal card
const revealGridCard: Card = {
  id: "radar_sweep",
  effects: [
    {
      intercepts: [{ component: 'Grid', operation: 'get_cell' }],
      transform: (query, result) => {
        // Reveal adjacent cells
        const adjacentCells = getAdjacentCells(query.position)
        return {
          ...result,
          cells: result.cells.map(cell =>
            adjacentCells.includes(cell.position)
              ? { ...cell, revealed: true }
              : cell
          )
        }
      },
      timing: 'after'
    }
  ]
}
```

### Component Query/Mutation System

All game state access goes through a **query/mutation pipeline** that cards can intercept:

```typescript
interface ComponentQuery {
  component: string    // e.g., 'Score', 'Grid', 'Question'
  operation: string    // e.g., 'get', 'add', 'update', 'remove'
  params: any          // Operation-specific parameters
}

interface ComponentMutation {
  component: string
  operation: string
  oldValue: any
  newValue: any
}

// Query pipeline
class ComponentSystem {
  private interceptors: CardEffect[] = []

  // Register card effects as interceptors
  registerCard(card: Card): void {
    this.interceptors.push(...card.effects)
  }

  // Execute a query with interceptor pipeline
  query<T>(query: ComponentQuery): T {
    let result = this.executeQuery(query)

    // Apply 'before' interceptors
    for (const effect of this.getInterceptors(query, 'before')) {
      result = effect.transform(query, result)
    }

    // Apply 'after' interceptors
    for (const effect of this.getInterceptors(query, 'after')) {
      result = effect.transform(query, result)
    }

    return result
  }

  // Execute a mutation with interceptor pipeline
  mutate(mutation: ComponentMutation): void {
    let { newValue } = mutation

    // Apply 'on_mutation' interceptors
    for (const effect of this.getInterceptors(mutation, 'on_mutation')) {
      newValue = effect.transform(mutation, newValue)
    }

    this.applyMutation({ ...mutation, newValue })
  }

  private getInterceptors(query: ComponentQuery | ComponentMutation, timing: string): CardEffect[] {
    return this.interceptors.filter(effect =>
      effect.timing === timing &&
      effect.intercepts.some(pattern =>
        pattern.component === query.component &&
        (pattern.operation === '*' || pattern.operation === query.operation)
      )
    )
  }
}
```

### Example: Card Effects Across Different Game Types

**Quiz Game - "Yaffa Drone Strike" (Eliminate Wrong Answer)**:
```yaml
card_id: yaffa_drone
name: "Yaffa Drone Strike"
effects:
  - intercepts:
      - component: Question
        operation: get_options
    transform: |
      // Remove one incorrect option
      const incorrect = result.filter(opt => !opt.correct)
      const toRemove = incorrect[Math.floor(Math.random() * incorrect.length)]
      return result.filter(opt => opt !== toRemove)
    timing: after
```

**Grid Game - "Radar Sweep" (Reveal Adjacent Cells)**:
```yaml
card_id: radar_sweep
name: "Radar Sweep"
effects:
  - intercepts:
      - component: Grid
        operation: get_cells
    transform: |
      // Reveal cells within 1 space of current position
      const { position } = query.params
      return result.map(cell => {
        const distance = Math.abs(cell.x - position.x) + Math.abs(cell.y - position.y)
        return distance <= 1 ? { ...cell, revealed: true } : cell
      })
    timing: after
```

**Deck Builder - "Resource Doubler" (Modify Resource Generation)**:
```yaml
card_id: resource_doubler
name: "Resource Doubler"
effects:
  - intercepts:
      - component: Resources
        operation: add
    transform: |
      // Double all resource gains this turn
      return {
        ...result,
        amount: result.amount * 2
      }
    timing: before
```

### Query Language for Card Effects

Cards can target specific component operations using a declarative syntax:

```yaml
# Match specific component + operation
intercepts:
  - component: Score
    operation: add

# Match all operations on a component
intercepts:
  - component: Timer
    operation: "*"

# Match multiple components
intercepts:
  - component: [Score, Timer, Question]
    operation: update

# Conditional interception
intercepts:
  - component: Grid
    operation: reveal_cell
    condition: "cell.x > 5 && cell.y < 3"  # Only intercept specific cells

# Parameterized interception
intercepts:
  - component: Question
    operation: get_options
    when:
      question_difficulty: hard  # Only intercept hard questions
```

### Effect Composition and Priority

Multiple cards can intercept the same query. Effects are applied in order:

```typescript
// Card 1: Add 2 points
effect1: (query, result) => ({ ...result, amount: result.amount + 2 })

// Card 2: Double points
effect2: (query, result) => ({ ...result, amount: result.amount * 2 })

// Applied in order:
// Initial: { amount: 5 }
// After effect1: { amount: 7 }
// After effect2: { amount: 14 }

// Cards can specify priority
interface CardEffect {
  priority: number  // Higher priority = applied first
  intercepts: ComponentQuery[]
  transform: Function
  timing: 'before' | 'after' | 'on_mutation'
}
```

### State Management with Components

The framework uses an Entity-Component-System (ECS) inspired architecture:

```typescript
// Entity: A unique game object
interface Entity {
  id: string
  components: Map<string, Component>
}

// Component: Pure data
interface Component {
  type: string
  data: any
}

// System: Logic that operates on components
interface System {
  update(entities: Entity[], deltaTime: number): void
  query(componentTypes: string[]): Entity[]
}

// Example: Quiz game state
const quizGame = {
  entities: [
    {
      id: 'current_question',
      components: new Map([
        ['Question', { text: '...', options: [...], points: 2 }],
        ['Timer', { remaining_ms: 30000, running: true }]
      ])
    },
    {
      id: 'player_score',
      components: new Map([
        ['Score', { current: 8, passing_grade: 20 }]
      ])
    },
    {
      id: 'active_card_yaffa',
      components: new Map([
        ['Card', { id: 'yaffa_drone', active: true }],
        ['Effect', { intercepts: [...], transform: ... }]
      ])
    }
  ]
}
```

---

## Framework Design Philosophy

### Data-Driven Everything

**Rule #1: All game-specific content must be external data, never hardcoded.**

```
❌ BAD - Hardcoded in engine:
function applyYaffaDroneEffect() {
  eliminateOneWrongAnswer();
}

✅ GOOD - Defined in data:
{
  "card_id": "yaffa_drone",
  "effects": [
    { "type": "ELIMINATE_WRONG_ANSWER", "count": 1 }
  ]
}

// Engine interprets effect data generically
function applyEffect(effect) {
  switch(effect.type) {
    case "ELIMINATE_WRONG_ANSWER":
      eliminateWrongAnswers(effect.count);
  }
}
```

### Content Creator Workflow

1. **Write quiz questions** (no programming):
   ```yaml
   questions:
     - id: q001
       text: "What year did X happen?"
       options:
         - {id: a, text: "1948", correct: true}
         - {id: b, text: "1967", correct: false}
   ```

2. **Define custom cards** (no programming):
   ```yaml
   cards:
     - id: my_custom_card
       name: "Super Helper"
       effects:
         - type: ADD_TIME
           seconds: 60
         - type: ADD_POINTS
           points: 5
   ```

3. **Configure game rules** (no programming):
   ```yaml
   game:
     passing_grade: 20
     timer_minutes: 15
     consensus_threshold: 2
   ```

4. **Compile** → Playable game with custom content

### Extensibility Requirements

The engine must support:

- **Custom card effects** without modifying engine code
- **Plugin system** for new effect types
- **Configurable game rules** (scoring, timing, voting)
- **Themeable UI** (colors, fonts, layouts)
- **Multiple question formats** (not just 4-choice)
- **Localization** (translate all text externally)

### Separation of Concerns

```
Engine responsibilities:
✅ Timer logic
✅ State management
✅ Event processing
✅ Twitch integration
✅ Effect interpretation
❌ Specific quiz topics
❌ Card artwork themes
❌ Hardcoded card names

Content responsibilities:
✅ Quiz questions
✅ Card definitions
✅ Game configuration
✅ Visual assets
✅ Audio files
❌ Game mechanics logic
```

---

## System Architecture

### Framework vs Implementation

**CRITICAL DISTINCTION**: CGQ is built in two layers:

1. **CGQ Engine (Framework Layer)** - Generic, reusable, content-agnostic
   - Core game mechanics (timer, state machine, scoring)
   - Card effect system (composable, extensible)
   - Twitch integration
   - Rendering pipeline
   - **NO hardcoded quiz content or cards**

2. **Game Implementations (Content Layer)** - Specific quiz games built using the engine
   - Quiz questions (JSON/YAML files)
   - Card definitions (JSON/YAML files)
   - Game configuration (rules, passing grade, timer settings)
   - Custom assets (images, sounds)
   - Compiled into playable game using the engine

### High-Level Components

```
┌─────────────────────────────────────────────────────────┐
│                    CONTENT LAYER                        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │ Quiz Data    │  │ Card Defs    │  │ Game Config  │  │
│  │ (JSON/YAML)  │  │ (JSON/YAML)  │  │ (JSON/YAML)  │  │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  │
└─────────┼──────────────────┼──────────────────┼─────────┘
          │                  │                  │
          └──────────────────┼──────────────────┘
                             │ Loaded by
┌─────────────────────────────▼─────────────────────────────┐
│                    ENGINE LAYER (CGQ Core)                │
│                                                            │
│  ┌─────────────────┐                                      │
│  │  Twitch Chat    │ ← User input (answers, votes, cmds) │
│  └────────┬────────┘                                      │
│           │                                               │
│           ▼                                               │
│  ┌─────────────────┐                                      │
│  │  Game Engine    │ ← Generic logic, state management   │
│  ├─────────────────┤                                      │
│  │ - Timer System  │ ← Framework code                     │
│  │ - Card System   │ ← Interprets card effect data        │
│  │ - Quiz Logic    │ ← Loads question data                │
│  │ - Score Tracker │ ← Configurable rules                 │
│  └────────┬────────┘                                      │
│           │                                               │
│           ▼                                               │
│  ┌─────────────────┐                                      │
│  │  Renderer/UI    │ ← Visual display (OBS/Browser)      │
│  └─────────────────┘                                      │
└───────────────────────────────────────────────────────────┘
```

### Technology Considerations

**Recommended Approach**: Build a data-driven engine with clear separation between framework and content

- **Language**: Rust, Go, or TypeScript (prefer native-compiled for single binary)
- **Architecture**: Event-driven system with plugin-based extensibility
- **Data Format**: JSON/YAML for all content (quiz, cards, configs)
- **Effect System**: Composable card effects defined in data, executed by engine
- **UI**: Web-based (HTML/CSS/JS) for OBS browser source integration
- **Twitch Integration**: IRC or Twitch API for chat monitoring
- **Build Process**: Content + Engine → Compiled Game Binary

---

## Game Modes

### 1. Normal Mode (Standard Quiz)
- All cards available from start
- Random card draws each question
- Fixed question set
- Complete when passing grade reached OR timer expires

### 2. Campaign Mode (Progressive Unlocking)
- Multiple levels with increasing difficulty
- Start with zero positive cards
- Earn currency from surplus points (points above passing grade)
- Store system for purchasing cards
- Store upgrades using specific card types
- Map progression through themed levels

**Campaign Features**:
- 5 levels per campaign
- Difficulty scaling
- Time-of-day progression (dawn → morning → noon → dusk → night)
- Environmental effects tied to time (more airstrikes at night, fewer ground attacks)
- Store upgrade tiers with better prices and more slots

### 3. Aftermath Mode (Post-Campaign)
- Unlocked after completing campaign
- Special themed cards
- Higher difficulty

---

## Core Features

### 1. Timer System

**Behavior**:
- Runs continuously during questions
- **PAUSES** when answer is displayed
- **RESUMES** when next question appears
- Can be modified by cards (+1 minute, -1 minute, etc.)
- Game ends when timer reaches zero (if passing grade not reached)

**Implementation Requirements**:
- Automatic pause/resume on state changes
- Card-based modifications must apply immediately
- Visual timer display with clear feedback

### 2. Question System

**Question Structure**:
```json
{
  "id": "q001",
  "question": "How is the Dahiya doctrine...",
  "options": [
    { "id": "a", "text": "...", "correct": false },
    { "id": "b", "text": "...", "correct": true },
    { "id": "c", "text": "...", "correct": false },
    { "id": "d", "text": "...", "correct": false }
  ],
  "points": 2,
  "explanation": "According to Article 25...",
  "source": "The Hague Convention of 1899"
}
```

**Randomization**:
- Question order randomized each playthrough
- Answer positions (A/B/C/D) randomized per question
- Prevents memorization-based cheating on replays

**Answer Submission**:
- Players type A, B, C, or D in chat
- Two matching answers = decision accepted
- Mismatched answers require tiebreaker
- Time pressure encourages coordination

### 3. Card System

**Card Types**:

1. **Resistance Cards** (Positive - Permanent)
   - Palestine Action: Bans specific negative cards when timer > 50%
   - Yaffa Drone Strike: Eliminates one incorrect answer
   - Operation Gates of Hell: Eliminates wrong option + 1 min + 2 points

2. **Palestinian Cards** (Positive - Permanent)
   - Francesca Albanese: Counters Hasbara cards, eliminates wrong option

3. **Politics Cards** (Positive - Permanent)
   - Donald Trump: +1 minute, +3 current answer points (costs 2 points to use)
   - Chinese Air Drop: Adds extra card slot on table

4. **IDF Cards** (Negative - Temporary)
   - Indiscriminate Bombing: Removes Palestinian/Resistance card from table
   - Reaper Drone: Various negative effects

5. **Event Cards** (Negative - Temporary)
   - 40 Beheaded Babies: -1 point value, -1 minute
   - Flour Massacre: Current question value becomes 1

**Card Behavior**:
- **Permanent cards**: Stay on table until used, occupy slots (max 4 slots)
- **Temporary cards**: Take effect immediately, disappear next question
- **Card deployment**: Requires 3 player votes (can be modified by cards)
- **Vote command**: `/use <card-name>` or `use <card-name>`
- **Vote tracking**: Visual indicator (pie chart) showing votes needed

**Card Drawing**:
- One card drawn per question (random)
- Cards can modify subsequent draws
- Some cards can ban other card types

### 4. Scoring System

**Points**:
- Each question has base point value (varies by importance)
- Cards can modify point values (+/- points)
- Passing grade typically 20 points
- Surplus points (in Campaign mode) become currency

**Win Conditions**:
- Reach passing grade before timer expires
- Fail if timer expires without passing grade

### 5. Store System (Campaign Mode Only)

**Store Levels**:

| Level | Slots | Resistance | Palestinian | Politics | Special | Required Cards |
|-------|-------|------------|-------------|----------|---------|----------------|
| 1     | 3     | 5          | 4           | 3        | varies  | -              |
| 2     | 4     | 4          | 3           | 2        | varies  | 6 Resistance   |
| 3     | 6     | 3          | 2           | 1        | varies  | More cards     |

**Store Mechanics**:
- Appears after each quiz completion
- Displays random selection of purchasable cards
- Players use surplus points as currency
- Purchased cards added to deck (not auto-deployed, just available in random draws)
- Store upgrades require depositing specific card types
- Upgraded stores have: more slots, lower prices, happier visuals

**Visual Progression**:
- Level 1: Sad girl with minimal products
- Level 2: Girl with helper, bicycle, more produce
- Level 3: Very happy girl, WFP bags, abundant products

---

## Twitch Integration

### Chat Commands

**Answer Submission**:
- Format: Single letter (A, B, C, or D)
- Two matching answers trigger answer lock

**Card Usage**:
- Format: `use <card-name>` or `/use <card-name>`
- Examples:
  - `use Palestine Action`
  - `use Yaffa Drone Strike`
- Requires 3 votes (or modified threshold)

**Vote Tracking**:
- Visual indicator on each card showing vote count
- Pie chart or counter showing "2/3 votes"
- Updates in real-time as votes come in
- Votes reset each question unless card is deployed

### Chat Parsing Requirements
- Case-insensitive matching
- Partial name matching (e.g., "yaffa" matches "Yaffa Drone Strike")
- Ignore duplicate votes from same user per question
- Clear vote state on question transition

---

## UI/UX Requirements

### Main Screen Layout

```
┌────────────────────────────────────────┐
│  Timer: 15:23                Score: 8  │
├────────────────────────────────────────┤
│                                        │
│  Question 3/15 [Worth 2 points]       │
│                                        │
│  What was the essential factor in...   │
│                                        │
│  A) Option text here                  │
│  B) Option text here                  │
│  C) Option text here                  │
│  D) Option text here                  │
│                                        │
├────────────────────────────────────────┤
│  Active Cards:                         │
│  [Card 1]  [Card 2]  [Card 3]  [Card 4]│
│  (2/3)              (1/3)              │
└────────────────────────────────────────┘
```

### Visual Feedback

**Answer Display**:
- Timer STOPS when answer shown
- Correct answer highlighted (green)
- Explanation text displayed
- Score updates with animation
- Brief pause before next question

**Card Effects**:
- Visual/audio feedback when card deployed
- Resistance cards: Gunfire sounds, muzzle flash effects
- Smooth animations for card entry/exit
- Vote indicators update smoothly

**Timer Display**:
- Clear, prominent timer
- Color changes as time gets critical (yellow → red)
- Modification animations when cards affect timer

### Campaign Mode UI

**Map Screen**:
- Visual map of Gaza (or relevant region)
- 5 levels marked on map
- Lighting changes with progression (dawn → night)
- Current level highlighted (green border)
- Completed levels marked

**Store Screen**:
- Visual representation of store keeper (evolves with upgrades)
- Card slots showing available purchases
- Price tags on each card
- Currency display (surplus points)
- Upgrade progress indicator

---

## Data Schema

### Quiz Configuration

```json
{
  "quiz_id": "hundred_years_war_pt1",
  "title": "The Hundred Years War on Palestine - Part 1",
  "description": "Based on Rashid Khalidi's book",
  "passing_grade": 20,
  "initial_timer_minutes": 16,
  "questions": [...],
  "cards": [...]
}
```

### Question Schema

```json
{
  "id": "q001",
  "text": "Question text here",
  "options": [
    { "id": "a", "text": "Option A", "correct": false },
    { "id": "b", "text": "Option B", "correct": true },
    { "id": "c", "text": "Option C", "correct": false },
    { "id": "d", "text": "Option D", "correct": false }
  ],
  "base_points": 2,
  "explanation": "Detailed explanation with sources",
  "tags": ["treaty", "1899", "hague_convention"]
}
```

### Card Schema

```json
{
  "id": "yaffa_drone",
  "name": "Yaffa Drone Strike",
  "type": "resistance",
  "permanence": "permanent",
  "effects": [
    { "type": "eliminate_wrong_answer", "count": 1 }
  ],
  "cost": 3,
  "vote_requirement": 3,
  "description": "Eliminates one incorrect answer",
  "counters": [],
  "countered_by": [],
  "visual": "yaffa_drone.png",
  "audio": "drone_strike.mp3"
}
```

### Campaign Configuration

```json
{
  "campaign_id": "hundred_years_war",
  "levels": [
    {
      "level": 1,
      "quiz_id": "hundred_years_war_pt1",
      "difficulty": 1,
      "time_of_day": "dawn",
      "negative_card_multiplier": 0.5,
      "map_position": { "x": 100, "y": 200 }
    }
  ],
  "store_levels": [
    {
      "level": 1,
      "slots": 3,
      "prices": { "resistance": 5, "palestinian": 4, "politics": 3 },
      "upgrade_requirement": { "type": "resistance", "count": 6 }
    }
  ]
}
```

---

## Persistence & User Data

### Storage Architecture

**Trait-Based Design**: Persistence layer uses traits/interfaces to allow swappable backends

```
┌─────────────────────────────────────────┐
│         Application Layer               │
│    (Campaign Manager, User Service)     │
└──────────────────┬──────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────────┐
│      StorageBackend Trait/Interface      │
│  - saveUserProgress()                    │
│  - loadUserProgress()                    │
│  - saveCardOwnership()                   │
│  - getUserCards()                        │
│  - updateCampaignState()                 │
└──────────────────┬──────────────────────┘
                   │
        ┌──────────┴──────────┬──────────┐
        │                     │          │
┌───────▼──────┐  ┌──────────▼───┐  ┌───▼──────┐
│   Supabase   │  │  PostgreSQL  │  │  SQLite  │
│ Implementation│  │Implementation│  │  Local   │
└──────────────┘  └──────────────┘  └──────────┘
```

### Initial Implementation: Supabase

**Why Supabase**:
- Free tier suitable for MVP
- Real-time subscriptions (future feature: live leaderboards)
- Built-in auth (Twitch OAuth integration possible)
- PostgreSQL-based (can migrate to self-hosted PG later)
- REST API + client libraries
- Row-level security for multi-tenant data

### Database Schema

#### Users Table
```sql
CREATE TABLE users (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  twitch_id VARCHAR(255) UNIQUE NOT NULL,
  twitch_username VARCHAR(255) NOT NULL,
  display_name VARCHAR(255),
  created_at TIMESTAMP DEFAULT NOW(),
  last_seen TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_users_twitch_id ON users(twitch_id);
```

#### Card Ownership Table
```sql
CREATE TABLE user_cards (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  user_id UUID REFERENCES users(id) ON DELETE CASCADE,
  card_id VARCHAR(255) NOT NULL, -- References card in content data
  quiz_id VARCHAR(255) NOT NULL, -- Which quiz this card belongs to
  acquisition_method VARCHAR(50) NOT NULL, -- 'earned', 'purchased', 'deposited'
  acquisition_date TIMESTAMP DEFAULT NOW(),
  is_deposited BOOLEAN DEFAULT FALSE, -- For store upgrades
  UNIQUE(user_id, card_id, quiz_id)
);

CREATE INDEX idx_user_cards_user_id ON user_cards(user_id);
CREATE INDEX idx_user_cards_quiz_id ON user_cards(user_id, quiz_id);
```

#### Campaign Progress Table
```sql
CREATE TABLE campaign_progress (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  user_id UUID REFERENCES users(id) ON DELETE CASCADE,
  quiz_id VARCHAR(255) NOT NULL, -- Which quiz/campaign
  current_level INTEGER DEFAULT 1,
  store_level INTEGER DEFAULT 1,
  currency INTEGER DEFAULT 0,
  completed_levels INTEGER[] DEFAULT '{}',
  updated_at TIMESTAMP DEFAULT NOW(),
  UNIQUE(user_id, quiz_id)
);

CREATE INDEX idx_campaign_progress_user_quiz ON campaign_progress(user_id, quiz_id);
```

#### Quiz Statistics Table
```sql
CREATE TABLE quiz_stats (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  user_id UUID REFERENCES users(id) ON DELETE CASCADE,
  quiz_id VARCHAR(255) NOT NULL,
  total_games INTEGER DEFAULT 0,
  wins INTEGER DEFAULT 0,
  losses INTEGER DEFAULT 0,
  total_questions_answered INTEGER DEFAULT 0,
  total_correct_answers INTEGER DEFAULT 0,
  best_score INTEGER DEFAULT 0,
  last_played TIMESTAMP,
  UNIQUE(user_id, quiz_id)
);

CREATE INDEX idx_quiz_stats_user_quiz ON quiz_stats(user_id, quiz_id);
```

#### Store Deposits Table (for tracking deposited cards)
```sql
CREATE TABLE store_deposits (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  user_id UUID REFERENCES users(id) ON DELETE CASCADE,
  quiz_id VARCHAR(255) NOT NULL,
  card_type VARCHAR(50) NOT NULL, -- 'resistance', 'palestinian', 'politics'
  count INTEGER DEFAULT 0,
  updated_at TIMESTAMP DEFAULT NOW(),
  UNIQUE(user_id, quiz_id, card_type)
);
```

### Trait/Interface Definition

**Rust Example**:
```rust
#[async_trait]
pub trait StorageBackend: Send + Sync {
    // User management
    async fn get_or_create_user(&self, twitch_id: &str, username: &str) -> Result<User>;
    async fn update_last_seen(&self, user_id: &str) -> Result<()>;

    // Card ownership
    async fn get_user_cards(&self, user_id: &str, quiz_id: &str) -> Result<Vec<UserCard>>;
    async fn add_card_to_user(
        &self,
        user_id: &str,
        card_id: &str,
        quiz_id: &str,
        method: AcquisitionMethod
    ) -> Result<UserCard>;
    async fn deposit_card_for_upgrade(
        &self,
        user_id: &str,
        card_id: &str,
        quiz_id: &str
    ) -> Result<()>;
    async fn user_owns_card(&self, user_id: &str, card_id: &str, quiz_id: &str) -> Result<bool>;

    // Campaign progress
    async fn get_campaign_progress(&self, user_id: &str, quiz_id: &str) -> Result<CampaignProgress>;
    async fn update_campaign_progress(&self, progress: &CampaignProgress) -> Result<()>;
    async fn complete_level(
        &self,
        user_id: &str,
        quiz_id: &str,
        level: i32,
        surplus_points: i32
    ) -> Result<()>;

    // Store management
    async fn get_store_deposits(&self, user_id: &str, quiz_id: &str) -> Result<StoreDeposits>;
    async fn increment_store_deposit(
        &self,
        user_id: &str,
        quiz_id: &str,
        card_type: CardType
    ) -> Result<i32>;
    async fn upgrade_store(&self, user_id: &str, quiz_id: &str) -> Result<()>;

    // Statistics
    async fn record_game_result(
        &self,
        user_id: &str,
        quiz_id: &str,
        won: bool,
        score: i32,
        questions_answered: i32,
        correct_answers: i32
    ) -> Result<()>;
    async fn get_stats(&self, user_id: &str, quiz_id: &str) -> Result<QuizStats>;
}
```

**TypeScript Example**:
```typescript
interface StorageBackend {
  // User management
  getOrCreateUser(twitchId: string, username: string): Promise<User>;
  updateLastSeen(userId: string): Promise<void>;

  // Card ownership
  getUserCards(userId: string, quizId: string): Promise<UserCard[]>;
  addCardToUser(
    userId: string,
    cardId: string,
    quizId: string,
    method: AcquisitionMethod
  ): Promise<UserCard>;
  depositCardForUpgrade(userId: string, cardId: string, quizId: string): Promise<void>;
  userOwnsCard(userId: string, cardId: string, quizId: string): Promise<boolean>;

  // Campaign progress
  getCampaignProgress(userId: string, quizId: string): Promise<CampaignProgress>;
  updateCampaignProgress(progress: CampaignProgress): Promise<void>;
  completeLevel(userId: string, quizId: string, level: number, surplusPoints: number): Promise<void>;

  // Store management
  getStoreDeposits(userId: string, quizId: string): Promise<StoreDeposits>;
  incrementStoreDeposit(userId: string, quizId: string, cardType: CardType): Promise<number>;
  upgradeStore(userId: string, quizId: string): Promise<void>;

  // Statistics
  recordGameResult(
    userId: string,
    quizId: string,
    won: boolean,
    score: number,
    questionsAnswered: number,
    correctAnswers: number
  ): Promise<void>;
  getStats(userId: string, quizId: string): Promise<QuizStats>;
}
```

### User Identification

**Twitch Chat Integration**:
- Extract Twitch user ID from chat messages (IRC tags)
- Create/update user record on first interaction
- Associate all card ownership and progress with Twitch ID
- No password/login required (Twitch handles auth)

**Example Flow**:
```
1. User types "A" in Twitch chat
2. Engine extracts Twitch ID from IRC tags
3. Engine calls storage.getOrCreateUser(twitchId, username)
4. User record created/retrieved
5. When card earned, storage.addCardToUser(userId, cardId, ...)
6. Progress saved automatically
```

### Data Models

```typescript
interface User {
  id: string;
  twitchId: string;
  twitchUsername: string;
  displayName?: string;
  createdAt: Date;
  lastSeen: Date;
}

interface UserCard {
  id: string;
  userId: string;
  cardId: string; // References card definition in content
  quizId: string;
  acquisitionMethod: 'earned' | 'purchased' | 'deposited';
  acquisitionDate: Date;
  isDeposited: boolean;
}

interface CampaignProgress {
  id: string;
  userId: string;
  quizId: string;
  currentLevel: number;
  storeLevel: number;
  currency: number;
  completedLevels: number[];
  updatedAt: Date;
}

interface StoreDeposits {
  userId: string;
  quizId: string;
  resistance: number;
  palestinian: number;
  politics: number;
}

interface QuizStats {
  userId: string;
  quizId: string;
  totalGames: number;
  wins: number;
  losses: number;
  totalQuestionsAnswered: number;
  totalCorrectAnswers: number;
  bestScore: number;
  lastPlayed?: Date;
}

enum AcquisitionMethod {
  Earned = 'earned',      // From campaign level completion
  Purchased = 'purchased', // Bought in store
  Deposited = 'deposited'  // Deposited for store upgrade
}
```

### Migration Strategy

**Framework Abstraction**: Since storage is trait-based, users can:
1. **Start with Supabase** (free, easy setup)
2. **Migrate to self-hosted PostgreSQL** (same schema, different connection)
3. **Use SQLite for local development** (file-based, no server)
4. **Implement custom backends** (e.g., MongoDB, DynamoDB)

**Migration Process**:
```bash
# Export from Supabase
cgq-cli export-data --from supabase --to backup.json

# Import to PostgreSQL
cgq-cli import-data --from backup.json --to postgresql://...

# Update config
# config.yml
storage:
  backend: postgresql
  connection_string: ${DATABASE_URL}
```

### Configuration

```yaml
# config.yml
storage:
  backend: supabase # or 'postgresql', 'sqlite', 'custom'

  supabase:
    url: ${SUPABASE_URL}
    anon_key: ${SUPABASE_ANON_KEY}

  postgresql:
    connection_string: ${DATABASE_URL}

  sqlite:
    file_path: ./data/cgq.db

  # Custom backend (plugin)
  custom:
    plugin_path: ./plugins/my_storage.so
```

### Privacy & Data Retention

**Minimal Data Collection**:
- Only store Twitch ID (public), username (public)
- No email, no passwords
- Game progress and stats only

**GDPR Compliance**:
- Users can request data deletion
- Export user data on request
- Clear data retention policy

**Data Deletion**:
```bash
cgq-cli delete-user-data --twitch-id <id>
```

---

## Game State Management

### State Machine

```
INIT → QUESTION_DISPLAY → WAITING_FOR_ANSWER → ANSWER_REVEALED →
(loop back to QUESTION_DISPLAY or → GAME_OVER)

GAME_OVER → STORE (campaign mode) → CAMPAIGN_MAP → INIT (next level)
```

### State Transitions

**INIT**:
- Load quiz data
- Initialize card deck
- Set initial timer
- Reset score

**QUESTION_DISPLAY**:
- Draw new card
- Randomize answer positions
- Start/resume timer
- Reset vote state

**WAITING_FOR_ANSWER**:
- Monitor chat for answers
- Monitor chat for card votes
- Process card deployments
- Check for timer expiration

**ANSWER_REVEALED**:
- Pause timer
- Show correct answer
- Display explanation
- Update score
- Apply any end-of-question effects

**GAME_OVER**:
- Stop timer
- Calculate final score
- Determine win/loss
- (Campaign) Calculate surplus points

**STORE** (campaign only):
- Generate random card selection
- Allow purchases
- Handle store upgrades
- Save progression

---

## Technical Requirements

### Core Systems

1. **Event System**
   - Card effects as events
   - Timer events
   - Chat events
   - State change events

2. **Card Effects Engine**
   - Composable effects
   - Priority/ordering system
   - Conflict resolution (e.g., multiple cards affecting same value)
   - Effect stacking/non-stacking rules

3. **Randomization System**
   - Seeded RNG for reproducibility (testing)
   - Weighted card draws
   - Question/answer shuffling

4. **Persistence System** (Campaign mode + user progression)
   - **Trait-based storage backend** for swappable implementations
   - **Initial implementation**: Supabase
   - **Alternative backends**: PostgreSQL, SQLite, File-based JSON
   - Store user progression between quiz instances
   - Track card ownership (earned/purchased)
   - Save campaign state (current level, store upgrades)
   - Level completion history
   - User authentication/identification

   **Key Requirement**: Users can quit mid-campaign and resume later

   **Data to Persist**:
   - User profile (Twitch ID, display name)
   - Owned cards (card_id, acquisition_date, acquisition_method)
   - Campaign progress (current_level, completed_levels)
   - Store state (store_level, deposited_cards, currency)
   - Quiz statistics (total_games, wins, losses, accuracy)

5. **Configuration Loader**
   - Hot-reload quizzes without restart
   - Validate quiz/card data
   - Support multiple quiz formats

### Non-Functional Requirements

- **Performance**: Minimal latency on chat input processing (<100ms)
- **Reliability**: Auto-recovery from disconnections
- **Scalability**: Support multiple concurrent quiz instances
- **Accessibility**: Screen reader support, colorblind-friendly design
- **Modularity**: Easy to add new quiz topics, cards, effects

---

## Development Phases

**IMPORTANT**: Each phase builds the **framework/engine**, not just a single quiz. The Palestinian history quiz serves as the reference implementation to validate the framework.

### Phase 1: Core Engine (MVP)
**Goal**: Build generic game engine with data loading capabilities

**Engine Development**:
- [ ] Event-driven architecture foundation
- [ ] Configuration loader (JSON/YAML parser)
- [ ] Question display engine (generic, content-agnostic)
- [ ] Answer processing system (configurable consensus rules)
- [ ] Timer system with pause/resume
- [ ] Basic card effect interpreter (data-driven)
- [ ] Twitch chat integration (command parsing)
- [ ] Score tracking (configurable rules)
- [ ] Win/loss conditions (configurable thresholds)
- [ ] Simple web-based renderer

**Reference Implementation**:
- [ ] Palestinian history quiz questions (JSON/YAML)
- [ ] Basic card set (5-10 cards defined in data)
- [ ] Game configuration file

**Deliverable**:
- Generic CGQ engine that loads external content
- Working Palestinian quiz as proof-of-concept
- Documentation for creating new quizzes

**Success Criteria**:
- ✅ Engine has NO hardcoded quiz content
- ✅ Can load different quiz files without code changes
- ✅ Reduces manual intervention by 80%

### Phase 2: Full Card System
**Goal**: Complete effect system with all effect types

**Engine Development**:
- [ ] All core effect types (timer, points, elimination, banning, etc.)
- [ ] Card voting system (generic, configurable thresholds)
- [ ] Vote visualization engine
- [ ] Card countering/banning logic
- [ ] Card slot management (configurable max slots)
- [ ] Effect composition engine (stacking, conflicts)
- [ ] Plugin system for custom effects

**Reference Implementation**:
- [ ] Full Palestinian quiz card set (20+ cards in data)
- [ ] Audio/visual effects assets
- [ ] Card interaction testing

**Deliverable**:
- Fully-featured card effect framework
- Complete Palestinian quiz with all cards
- Card effect documentation/API

**Success Criteria**:
- ✅ Content creators can define custom cards without coding
- ✅ All card interactions work generically

### Phase 3: Campaign Mode + Persistence
**Goal**: Progression framework with persistent user data across sessions

**Engine Development - Persistence**:
- [ ] StorageBackend trait/interface definition
- [ ] Supabase implementation (initial backend)
- [ ] Database schema design and migrations
- [ ] User identification from Twitch chat (automatic)
- [ ] Card ownership tracking system
- [ ] Campaign progress persistence
- [ ] Store deposits tracking
- [ ] Quiz statistics recording
- [ ] Data export/import utilities
- [ ] Backend migration tools

**Engine Development - Campaign**:
- [ ] Campaign data structure and loader
- [ ] Level progression engine (with persistence)
- [ ] Store system (generic purchasing/upgrading)
- [ ] Campaign map renderer (configurable layouts)
- [ ] Difficulty scaling system
- [ ] Time-based visual effects engine

**Reference Implementation**:
- [ ] Palestinian quiz campaign (5 levels)
- [ ] Campaign map assets
- [ ] Store progression configuration
- [ ] Supabase database setup scripts
- [ ] Sample user data for testing

**Deliverable**:
- Campaign framework usable for any quiz topic
- Trait-based persistence layer with Supabase implementation
- Complete Palestinian quiz campaign with saved progress
- Database schema documentation
- Migration guide for other backends (PostgreSQL, SQLite)
- Campaign creation guide

**Success Criteria**:
- ✅ Can create campaigns with different themes/maps
- ✅ Store system works with any card set
- ✅ Users can quit and resume campaign progress
- ✅ Card ownership persists between sessions
- ✅ Can swap storage backend without changing game logic
- ✅ Multi-user support (multiple Twitch users can have separate progress)

### Phase 4: Visual Content Editor (CGQ Builder)
**Goal**: Visual editor for non-programmers to create games without writing code

**CRITICAL**: This is the key to adoption. Non-technical content creators must be able to:
- Create cards with complex interceptor logic visually
- Design game content for multiple game types (Quiz, Grid, etc.)
- Preview cards and content in real-time
- Export production-ready YAML/JSON

**Engine Development**:
- [ ] Schema validation for quiz/card/config files (JSON Schema)
- [ ] Content compiler (validates and packages content)
- [ ] Asset management system (upload, organize, reference)
- [ ] Hot-reload for development (watch files, auto-reload)
- [ ] Content API (REST/GraphQL for editor to communicate with engine)
- [ ] Preview mode (run game with test data from editor)

**Editor Application Development**:

**A. Universal Card Editor**:
- [ ] Visual card designer (drag-drop UI)
- [ ] Interceptor builder (no-code effect creation)
  - [ ] Component selector dropdown (Score, Timer, Grid, Question, etc.)
  - [ ] Operation selector (get, add, update, reveal, etc.)
  - [ ] Transform function builder (visual scripting or template library)
  - [ ] Timing selector (before/after/on_mutation)
  - [ ] Condition builder (visual expression editor)
  - [ ] Priority slider
- [ ] Effect template library (common patterns: "Double Points", "Add Time", etc.)
- [ ] Real-time preview (see card effect in action)
- [ ] Card artwork uploader
- [ ] Sound effect selector
- [ ] Validation and error checking
- [ ] Export to YAML/JSON

**B. Game Type-Specific Content Editors**:

**Quiz Editor**:
- [ ] Question list manager (CRUD operations)
- [ ] Rich text editor for question/answer text
- [ ] Answer option builder (A/B/C/D with correct flag)
- [ ] Point value slider
- [ ] Explanation text editor
- [ ] Source citation field
- [ ] Question tags/categories
- [ ] Bulk import from CSV/spreadsheet
- [ ] Preview mode (see question as players will)

**Grid Editor** (for Battleship-style games):
- [ ] Visual grid designer (drag-drop ships, obstacles)
- [ ] Grid size configurator
- [ ] Ship/object placement tool
- [ ] Cell properties editor (occupied, revealed, special)
- [ ] Preview mode (test grid layout)

**Deck Builder Editor** (for future deck-building games):
- [ ] Card deck designer
- [ ] Card stats editor
- [ ] Resource definition
- [ ] Deck list manager

**C. Campaign Designer**:
- [ ] Visual map editor (place levels on map image)
- [ ] Level configuration (difficulty, time-of-day, quiz assignment)
- [ ] Store progression designer
  - [ ] Store level configurator (slots, prices)
  - [ ] Upgrade requirements editor
  - [ ] Visual asset uploader (store keeper images)
- [ ] Campaign flow tester

**D. Universal Features**:
- [ ] Project manager (create/open/save projects)
- [ ] Asset library (manage images, sounds, data files)
- [ ] Game configuration editor (timer, passing grade, rules)
- [ ] Preview/playtest mode (run game in editor)
- [ ] Export wizard (generate game bundle)
- [ ] Import/export content packs
- [ ] Version control integration (Git)
- [ ] Collaboration features (share projects, comments)
- [ ] Template gallery (start from examples)

**Technology Stack for Editor**:
- **Platform**: Web-based (Electron for desktop wrapper)
- **Frontend**: React/Vue + visual editor library (e.g., React Flow for node-based)
- **Backend**: CGQ engine API
- **Database**: IndexedDB (local) or cloud storage (optional)
- **Code Editor**: Monaco Editor (for advanced users who want to write transforms)

**Editor UI Mockup Concepts**:

```
┌────────────────────────────────────────────────────────┐
│  CGQ Builder                         [Project: Quiz]   │
├──────────┬─────────────────────────────────────────────┤
│          │  Card Editor: "Yaffa Drone Strike"         │
│ Projects │                                             │
│ Cards    │  ┌───────────────────────────────────┐     │
│ Questions│  │ Name: Yaffa Drone Strike          │     │
│ Config   │  │ Type: ○ Resistance ○ Palestinian   │     │
│ Assets   │  │       ○ Politics   ○ IDF          │     │
│ Preview  │  └───────────────────────────────────┘     │
│          │                                             │
│ [+ New   │  Effects (1):                               │
│  Card]   │  ┌─────────────────────────────────────┐   │
│          │  │ Effect #1: Eliminate Wrong Answer   │   │
│          │  │                                     │   │
│          │  │ Intercepts:                         │   │
│          │  │  Component: [Question ▼]            │   │
│          │  │  Operation: [get_options ▼]         │   │
│          │  │                                     │   │
│          │  │ Transform: [Visual Builder ▼]       │   │
│          │  │  ┌──────────────────────────────┐   │   │
│          │  │  │ Filter: incorrect options    │   │   │
│          │  │  │ Select: random               │   │   │
│          │  │  │ Action: remove               │   │   │
│          │  │  └──────────────────────────────┘   │   │
│          │  │                                     │   │
│          │  │ Timing: ○ Before ● After ○ Mutation│   │
│          │  │ Priority: [100]────────────         │   │
│          │  │ Duration: [One-time ▼]              │   │
│          │  └─────────────────────────────────────┘   │
│          │                                             │
│          │  [+ Add Effect]                             │
│          │                                             │
│          │  Artwork:  [Upload Image]  [Preview]        │
│          │  Sound:    [Select Audio]  [🔊 Play]        │
│          │                                             │
│          │  [Cancel]  [Save Card]  [Test in Preview]   │
└──────────┴─────────────────────────────────────────────┘
```

**Visual Transform Builder**:

Instead of writing JavaScript, users select from templates:

```
Transform Type: [Modify Collection ▼]

┌────────────────────────────────────┐
│ Input: options (array)             │
│                                    │
│ Step 1: Filter                     │
│   Keep: [incorrect options ▼]      │
│                                    │
│ Step 2: Select                     │
│   Method: [random ▼]               │
│   Count: [1]                       │
│                                    │
│ Step 3: Action                     │
│   Action: [remove from list ▼]     │
│                                    │
│ Output: modified options           │
└────────────────────────────────────┘

[Generate Code]  <-- Shows resulting transform function
```

**Advanced users** can switch to code view and write JavaScript directly.

**Quiz Question Editor**:

```
┌────────────────────────────────────────────────────────┐
│  Question Editor                                       │
├────────────────────────────────────────────────────────┤
│  Question Text:                                        │
│  ┌──────────────────────────────────────────────────┐ │
│  │ How is the Dahiya doctrine a violation of the    │ │
│  │ 1899 annex of the Hague Convention?              │ │
│  └──────────────────────────────────────────────────┘ │
│                                                        │
│  Options:                                              │
│  ○ A: [It targets civilians                    ] ✓    │
│  ○ B: [It violates territorial sovereignty     ] ✗    │
│  ○ C: [It uses banned weapons                  ] ✗    │
│  ○ D: [It ignores diplomatic channels          ] ✗    │
│                                                        │
│  Points: [2]────────                                   │
│                                                        │
│  Explanation:                                          │
│  ┌──────────────────────────────────────────────────┐ │
│  │ According to Article 25 of the Hague Convention  │ │
│  │ ...                                              │ │
│  └──────────────────────────────────────────────────┘ │
│                                                        │
│  Source: [The Hague Convention of 1899]               │
│  Tags: [treaty] [1899] [war_crimes]                   │
│                                                        │
│  [Delete]  [Duplicate]  [Save]                         │
└────────────────────────────────────────────────────────┘
```

**Documentation**:
- [ ] Comprehensive framework docs
- [ ] Editor user guide with screenshots
- [ ] Video tutorials: "Create Your First Card"
- [ ] Video tutorials: "Build a Quiz in 10 Minutes"
- [ ] Card effect template gallery
- [ ] Example projects (history, science, etc.)
- [ ] Troubleshooting guide
- [ ] API reference for advanced users

**Deliverable**:
- Complete content creation toolkit
- "CGQ Builder" application
- Multiple example quiz packs

**Success Criteria**:
- ✅ Non-programmers can create full quizzes
- ✅ Community can share quiz packs
- ✅ Framework is well-documented

### Phase 5: Polish & Enhancement
**Goal**: Production-ready framework distribution

**Engine Development**:
- [ ] Performance optimization
- [ ] Advanced visual effects system
- [ ] Accessibility framework
- [ ] Localization system (i18n)
- [ ] Analytics/telemetry framework
- [ ] Plugin marketplace infrastructure

**Ecosystem Development**:
- [ ] Multiple official quiz packs (history, science, literature)
- [ ] Community quiz repository
- [ ] Quiz sharing/distribution platform
- [ ] Mobile app support
- [ ] Cloud hosting option

**Deliverable**:
- Professional, distributable CGQ framework
- Library of diverse quiz content
- Active creator community

**Success Criteria**:
- ✅ Framework is used by multiple content creators
- ✅ Quizzes exist on various topics
- ✅ Active community contributing content

---

## Open Questions / Decisions Needed

1. **Platform**: Web app vs native app vs hybrid?
2. **Hosting**: Self-hosted vs cloud? Twitch extension?
3. **Multiplayer**: Single streamer or support for multiple concurrent instances?
4. **Monetization**: Free/open source vs paid/premium features?
5. **Content moderation**: How to handle user-generated quizzes?
6. **Language support**: I18n from start or later?
7. **Offline mode**: Support for non-Twitch usage?
8. **Card balancing**: Playtesting process for new cards?
9. **Anti-cheat**: Prevent the same user voting multiple times with alt accounts?

---

## Success Metrics

**For MVP**:
- Quiz runs without manual intervention
- Timer management is automatic
- Chat answers are processed correctly
- Win/loss determination is accurate

**For Full Release**:
- Host can run quiz with <5% manual actions
- Players report engagement is high
- Content creators adopt for their own topics
- Community creates custom quizzes

---

## Resources Needed

**Team**:
- 2-3 programmers (backend/frontend)
- 1 UI/UX designer
- 1 game artist (card illustrations, visual effects)
- 1 sound designer (optional, can use stock audio initially)

**Infrastructure**:
- Git repository
- CI/CD pipeline
- Hosting for web app (if applicable)
- Twitch developer account

**Tools**:
- Quiz content (already exists: multiple books/topics prepared)
- Card artwork (some exists, more needed)
- Audio assets (gunfire, effects, music)

---

## Next Steps

1. **Define technical stack** - Choose language, frameworks, deployment strategy
2. **Create proof of concept** - Basic question → answer → next question loop
3. **Implement Twitch integration** - Chat parsing and response
4. **Build timer system** - Automatic pause/resume
5. **Prototype card system** - 3-5 basic cards with effects
6. **Iterate based on playtesting**

---

## References

- Transcript: `share/transcripts/2026-01-15_NA_Kairos_Rev_ (live) 2026-01-15 10_06.txt`
- Existing quiz content: Multiple quiz packs based on various books
- Card artwork: In development
- Target platform: Twitch streaming integration (OBS browser source)

---

*Document created: 2026-01-15*
*Based on: Kairos Rev live stream transcript*
*Project codename: CGQ (Card Game Quiz)*
