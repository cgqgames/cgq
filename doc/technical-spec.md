# CGQ Framework - Technical Specification

## Architecture Overview

### Critical Distinction: Framework vs Game

**CGQ is NOT a quiz application. CGQ is a framework/engine for building card-based interactive games.**

This document describes:
1. **CGQ Engine** - The reusable, game-type-agnostic framework (what we're building)
2. **Game Type Plugins** - Quiz, Grid, Deck-builder, etc. (implemented as traits)
3. **Reference Implementation** - The Palestinian history quiz (validates the framework)

### System Design Philosophy

**Core Principles:**

1. **Everything game-specific must be external data or plugins, never hardcoded**
2. **Game types are pluggable** - Support Quiz, Grid, Deck-builder, and custom types
3. **Cards are universal** - Same card system works across all game types via interceptors
4. **Component-based architecture** - Game state is composed of reusable components
5. **Interceptor pattern** - Cards hook into component queries/mutations

Build a **generic CGQ engine** that:
- **Game-type-agnostic**: Supports multiple game types via trait system
- **Format-agnostic**: Content in JSON/YAML, not hardcoded structures
- **Source-agnostic**: Data can come from files, APIs, databases
- **Extensible**: Plugin system for new game types and card effects
- **Protocol-based**: Clean interfaces between components
- **Data-driven**: All game rules, cards, content loaded from configuration
- **Themeable**: Visual design/assets separate from engine logic

**Anti-patterns to avoid**:

```typescript
// ❌ BAD - Hardcoded game type in engine
if (gameType === "quiz") {
  processQuizAnswer(input);
} else if (gameType === "grid") {
  processGridMove(input);
}

// ✅ GOOD - Game type plugin handles its own logic
const gameType = loadGameType(config.game_type);
gameType.process_input(state, input);

// ❌ BAD - Hardcoded card effect
if (card.id === "yaffa_drone") {
  eliminateOneWrongAnswer();
}

// ✅ GOOD - Interceptor-based effect
card.effects.forEach(effect => {
  componentSystem.registerInterceptor(effect);
});
// Effect automatically intercepts relevant component queries
```

### Component Architecture

**Two-Layer Design**: Engine (Framework) + Content (Data)

```
┌────────────────────────────────────────────────────────────┐
│                     CONTENT LAYER                          │
│                   (Game-Specific Data)                     │
│                                                            │
│  ┌─────────────┐  ┌─────────────┐  ┌──────────────┐      │
│  │ Questions   │  │ Cards       │  │ Config       │      │
│  │ (JSON/YAML) │  │ (JSON/YAML) │  │ (JSON/YAML)  │      │
│  │             │  │             │  │              │      │
│  │ - id        │  │ - effects   │  │ - timer      │      │
│  │ - text      │  │ - type      │  │ - scoring    │      │
│  │ - options   │  │ - cost      │  │ - voting     │      │
│  └──────┬──────┘  └──────┬──────┘  └──────┬───────┘      │
│         │                │                 │              │
└─────────┼────────────────┼─────────────────┼──────────────┘
          │                │                 │
          └────────────────┼─────────────────┘
                           │ Loads into
┌──────────────────────────┼────────────────────────────────┐
│                    ENGINE LAYER                            │
│                (Framework - Reusable)                      │
│                                                            │
│  ┌──────────────────────────────────────────┐             │
│  │        Presentation Layer                │             │
│  │  ┌──────────┐  ┌──────────┐  ┌────────┐ │             │
│  │  │OBS/Browse│  │CLI Display│  │Web UI  │ │             │
│  │  └────┬─────┘  └────┬──────┘  └───┬────┘ │             │
│  └───────┼─────────────┼─────────────┼──────┘             │
│          └─────────────┼─────────────┘                    │
│                        │                                  │
│  ┌─────────────────────▼──────────────────┐               │
│  │      Application Layer                 │               │
│  │   ┌────────────────────────┐           │               │
│  │   │   Game Controller      │           │               │
│  │   │ (Orchestrates Engine)  │           │               │
│  │   └──────────┬─────────────┘           │               │
│  │   ┌──────────┼──────────────────┐      │               │
│  │   │          │                  │      │               │
│  │ ┌─▼──┐ ┌────▼───┐ ┌────▼───┐ ┌─▼────┐ │               │
│  │ │Timer│ │Card Sys│ │Quiz Eng│ │Score │ │               │
│  │ │ Sys │ │Manager │ │  (gen) │ │Track │ │               │
│  │ └─────┘ └────────┘ └────────┘ └──────┘ │               │
│  └──────────────────┬────────────────────┘                │
│                     │                                     │
│  ┌──────────────────▼────────────────────┐                │
│  │      Integration Layer                │                │
│  │  ┌────────┐ ┌──────────┐ ┌─────────┐ │                │
│  │  │ Twitch │ │  Config  │ │  Event  │ │                │
│  │  │ Client │ │  Loader  │ │   Bus   │ │                │
│  │  └────────┘ └──────────┘ └─────────┘ │                │
│  └────────────────────────────────────────                │
│                                                            │
│  KEY PRINCIPLE:                                           │
│  ✅ Engine components = Generic, reusable                 │
│  ✅ Content data = Game-specific                          │
│  ❌ NO hardcoded quiz content in engine                   │
└────────────────────────────────────────────────────────────┘
```

---

## Game Type System

### GameType Trait

All game types implement a common trait/interface:

```rust
#[async_trait]
pub trait GameType: Send + Sync {
    // Type definitions
    type State: Clone + Serialize + Deserialize;
    type Input;
    type Config;

    // Lifecycle
    fn initialize(&self, config: &Self::Config) -> Self::State;
    fn update(&mut self, state: &mut Self::State, delta_time: f64);
    fn check_win_condition(&self, state: &Self::State) -> Option<GameResult>;

    // Input processing
    fn process_input(&mut self, state: &mut Self::State, input: Self::Input) -> Result<()>;

    // Component access
    fn get_components(&self, state: &Self::State) -> Vec<Component>;
    fn query_component(&self, state: &Self::State, query: &ComponentQuery) -> QueryResult;
    fn mutate_component(&mut self, state: &mut Self::State, mutation: &ComponentMutation);

    // Metadata
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn required_components(&self) -> Vec<String>;
}
```

**TypeScript Interface**:
```typescript
interface GameType<TState, TInput, TConfig> {
  // Lifecycle
  initialize(config: TConfig): TState
  update(state: TState, deltaTime: number): void
  checkWinCondition(state: TState): GameResult | null

  // Input processing
  processInput(state: TState, input: TInput): void

  // Component access
  getComponents(state: TState): Component[]
  queryComponent(state: TState, query: ComponentQuery): any
  mutateComponent(state: TState, mutation: ComponentMutation): void

  // Metadata
  name: string
  description: string
  requiredComponents: string[]
}
```

### Built-in Game Types

#### 1. Quiz Game Type

```typescript
class QuizGameType implements GameType<QuizState, QuizInput, QuizConfig> {
  name = "Quiz"
  description = "Multiple-choice question-answering game"
  requiredComponents = ["Question", "Score", "Timer", "Answer"]

  initialize(config: QuizConfig): QuizState {
    return {
      currentQuestionIndex: 0,
      questions: shuffle(config.questions),
      score: 0,
      passingGrade: config.passingGrade,
      timer: config.initialTimerSeconds * 1000,
      activeCards: []
    }
  }

  processInput(state: QuizState, input: QuizInput): void {
    if (input.type === 'ANSWER_SUBMITTED') {
      const question = state.questions[state.currentQuestionIndex]
      const correct = question.options.find(o => o.id === input.answer)?.correct

      if (correct) {
        // Trigger score mutation (interceptors can modify this)
        this.mutateComponent(state, {
          component: 'Score',
          operation: 'add',
          oldValue: state.score,
          newValue: state.score + question.points
        })
      }

      state.currentQuestionIndex++
    }
  }

  queryComponent(state: QuizState, query: ComponentQuery): any {
    // All queries go through interceptor pipeline
    switch (query.component) {
      case 'Question':
        if (query.operation === 'get_current') {
          return state.questions[state.currentQuestionIndex]
        }
        if (query.operation === 'get_options') {
          return state.questions[state.currentQuestionIndex].options
        }
        break

      case 'Score':
        if (query.operation === 'get') {
          return state.score
        }
        break

      case 'Timer':
        if (query.operation === 'get_remaining') {
          return state.timer
        }
        break
    }
  }

  mutateComponent(state: QuizState, mutation: ComponentMutation): void {
    // All mutations go through interceptor pipeline
    switch (mutation.component) {
      case 'Score':
        if (mutation.operation === 'add') {
          state.score = mutation.newValue
        }
        break

      case 'Timer':
        if (mutation.operation === 'adjust') {
          state.timer += mutation.params.delta
        }
        break
    }
  }
}
```

#### 2. Grid Game Type (Battleship-style)

```typescript
class GridGameType implements GameType<GridState, GridInput, GridConfig> {
  name = "Grid"
  description = "Grid-based positioning and revelation game"
  requiredComponents = ["Grid", "Position", "Ship", "Score"]

  initialize(config: GridConfig): GridState {
    return {
      grid: createGrid(config.width, config.height),
      ships: placeShips(config.ships, config.width, config.height),
      revealedCells: new Set(),
      score: 0,
      movesRemaining: config.maxMoves
    }
  }

  processInput(state: GridState, input: GridInput): void {
    if (input.type === 'CELL_SELECTED') {
      const { x, y } = input.position

      // Query cell (interceptors can reveal adjacent cells, etc.)
      const cell = this.queryComponent(state, {
        component: 'Grid',
        operation: 'get_cell',
        params: { x, y }
      })

      // Mutate cell state
      this.mutateComponent(state, {
        component: 'Position',
        operation: 'reveal',
        params: { x, y }
      })

      state.revealedCells.add(`${x},${y}`)
      state.movesRemaining--
    }
  }

  queryComponent(state: GridState, query: ComponentQuery): any {
    switch (query.component) {
      case 'Grid':
        if (query.operation === 'get_cell') {
          const { x, y } = query.params
          return state.grid[y][x]
        }
        if (query.operation === 'get_cells') {
          return state.grid.flat()
        }
        break

      case 'Position':
        if (query.operation === 'is_revealed') {
          const { x, y } = query.params
          return state.revealedCells.has(`${x},${y}`)
        }
        break
    }
  }
}
```

### Game Type Configuration

```yaml
# Quiz game config
game_type: quiz
config:
  passing_grade: 20
  initial_timer_seconds: 960  # 16 minutes
  questions_file: ./questions/part1.yml
  randomize_questions: true
  randomize_options: true

# Grid game config
game_type: grid
config:
  width: 10
  height: 10
  ships:
    - size: 5
      count: 1
    - size: 4
      count: 2
    - size: 3
      count: 3
  max_moves: 50
  questions_file: ./questions/naval.yml  # Answer correctly to get moves
```

---

## Component/Interceptor Architecture

### Component Query System

All game state access goes through a **query/mutation pipeline**:

```typescript
interface ComponentQuery {
  component: string       // e.g., 'Score', 'Grid', 'Question'
  operation: string       // e.g., 'get', 'add', 'update'
  params?: any            // Operation-specific parameters
}

interface ComponentMutation {
  component: string
  operation: string
  oldValue?: any
  newValue: any
  params?: any
}

interface QueryResult {
  value: any
  metadata?: {
    source: string
    timestamp: number
    intercepted_by: string[]
  }
}
```

### ComponentSystem Implementation

```typescript
class ComponentSystem {
  private gameType: GameType<any, any, any>
  private interceptors: Map<string, CardEffect[]> = new Map()

  constructor(gameType: GameType<any, any, any>) {
    this.gameType = gameType
  }

  // Register a card's effects as interceptors
  registerCard(card: Card): void {
    for (const effect of card.effects) {
      for (const pattern of effect.intercepts) {
        const key = `${pattern.component}:${pattern.operation}`
        if (!this.interceptors.has(key)) {
          this.interceptors.set(key, [])
        }
        this.interceptors.get(key)!.push(effect)
      }
    }

    // Sort by priority (higher first)
    this.interceptors.forEach(effects => {
      effects.sort((a, b) => (b.priority || 0) - (a.priority || 0))
    })
  }

  // Execute query with interceptor pipeline
  query(state: any, query: ComponentQuery): any {
    // Get base result from game type
    let result = this.gameType.queryComponent(state, query)

    // Apply 'before' interceptors
    result = this.applyInterceptors(query, result, 'before')

    // Apply 'after' interceptors
    result = this.applyInterceptors(query, result, 'after')

    return result
  }

  // Execute mutation with interceptor pipeline
  mutate(state: any, mutation: ComponentMutation): void {
    let { newValue } = mutation

    // Apply 'on_mutation' interceptors
    newValue = this.applyMutationInterceptors(mutation, newValue)

    // Apply to game state
    this.gameType.mutateComponent(state, { ...mutation, newValue })
  }

  private applyInterceptors(query: ComponentQuery, result: any, timing: string): any {
    const interceptors = this.getInterceptors(query, timing)

    for (const effect of interceptors) {
      // Check conditions if any
      if (effect.condition && !this.evaluateCondition(effect.condition, query, result)) {
        continue
      }

      // Apply transform
      try {
        result = effect.transform(query, result)
      } catch (error) {
        console.error(`Error in interceptor ${effect.id}:`, error)
      }
    }

    return result
  }

  private getInterceptors(query: ComponentQuery, timing: string): CardEffect[] {
    const exactKey = `${query.component}:${query.operation}`
    const wildcardKey = `${query.component}:*`

    const exact = this.interceptors.get(exactKey) || []
    const wildcard = this.interceptors.get(wildcardKey) || []

    return [...exact, ...wildcard].filter(e => e.timing === timing)
  }

  private evaluateCondition(condition: string, query: ComponentQuery, result: any): boolean {
    // Simple expression evaluator (in real impl, use a safe eval library)
    try {
      const func = new Function('query', 'result', `return ${condition}`)
      return func(query, result)
    } catch {
      return false
    }
  }
}
```

### Card Effect Definition

```typescript
interface CardEffect {
  id: string
  priority?: number              // Higher = applied first (default: 0)
  intercepts: InterceptPattern[]
  transform: (query: ComponentQuery, result: any) => any
  timing: 'before' | 'after' | 'on_mutation'
  condition?: string             // JavaScript expression to evaluate
  duration?: 'permanent' | 'one_time' | 'turns' | 'duration_ms'
}

interface InterceptPattern {
  component: string | string[]   // Component name(s)
  operation: string              // Operation name or '*' for all
  when?: Record<string, any>     // Conditional parameters
}
```

### Card Definition in YAML

```yaml
# Quiz game card
card_id: yaffa_drone_strike
name: "Yaffa Drone Strike"
type: resistance
effects:
  - id: eliminate_wrong_answer
    priority: 100
    intercepts:
      - component: Question
        operation: get_options
    transform: |
      const incorrect = result.filter(opt => !opt.correct);
      if (incorrect.length === 0) return result;
      const idx = Math.floor(Math.random() * incorrect.length);
      return result.filter((_, i) => result.indexOf(incorrect[idx]) !== i);
    timing: after
    duration: one_time

# Grid game card
card_id: radar_sweep
name: "Radar Sweep"
type: utility
effects:
  - id: reveal_adjacent
    priority: 50
    intercepts:
      - component: Grid
        operation: get_cells
    transform: |
      const { position } = query.params;
      return result.map(cell => {
        const dx = Math.abs(cell.x - position.x);
        const dy = Math.abs(cell.y - position.y);
        const distance = dx + dy;
        return distance <= 1 ? { ...cell, revealed: true } : cell;
      });
    timing: after
    duration: one_time

# Universal card (works in any game type)
card_id: time_warp
name: "Time Warp"
type: temporal
effects:
  - id: extend_timer
    intercepts:
      - component: Timer
        operation: get_remaining
    transform: |
      return result + 60000; // Add 1 minute
    timing: after
    duration: one_time
```

### Effect Execution Example

```typescript
// Setup
const gameType = new QuizGameType()
const state = gameType.initialize(config)
const componentSystem = new ComponentSystem(gameType)

// Register cards
const yaffaCard = loadCard('yaffa_drone_strike.yml')
componentSystem.registerCard(yaffaCard)

// Query with interception
const question = componentSystem.query(state, {
  component: 'Question',
  operation: 'get_current'
})
// Returns normal question

const options = componentSystem.query(state, {
  component: 'Question',
  operation: 'get_options'
})
// Returns options with one incorrect option removed (yaffa card intercepts)

// Mutation with interception
componentSystem.mutate(state, {
  component: 'Score',
  operation: 'add',
  oldValue: 10,
  newValue: 13  // Base +3 points
})
// If "double_points" card is active, mutation interceptor changes newValue to 16
```

---

## Core Systems

### 1. Event System

All components communicate via events. This enables loose coupling and easy testing.

#### Event Types

```typescript
// Core events
type GameEvent =
  | { type: 'GAME_STARTED'; quiz_id: string }
  | { type: 'QUESTION_DISPLAYED'; question: Question }
  | { type: 'ANSWER_SUBMITTED'; player: string; answer: string }
  | { type: 'ANSWER_LOCKED'; answer: string }
  | { type: 'ANSWER_CORRECT'; points_awarded: number }
  | { type: 'ANSWER_INCORRECT' }
  | { type: 'QUESTION_COMPLETED' }
  | { type: 'GAME_OVER'; won: boolean; final_score: number }

// Timer events
  | { type: 'TIMER_STARTED'; duration_seconds: number }
  | { type: 'TIMER_PAUSED'; remaining_seconds: number }
  | { type: 'TIMER_RESUMED'; remaining_seconds: number }
  | { type: 'TIMER_EXPIRED' }
  | { type: 'TIMER_MODIFIED'; delta_seconds: number }

// Card events
  | { type: 'CARD_DRAWN'; card: Card }
  | { type: 'CARD_VOTE_CAST'; player: string; card_id: string }
  | { type: 'CARD_DEPLOYED'; card: Card }
  | { type: 'CARD_EFFECT_APPLIED'; effect: CardEffect }
  | { type: 'CARD_REMOVED'; card_id: string }

// Chat events
  | { type: 'CHAT_MESSAGE'; user: string; message: string }
```

#### Event Bus Implementation

```typescript
interface EventBus {
  publish(event: GameEvent): void
  subscribe(eventType: string, handler: (event: GameEvent) => void): Unsubscribe
  subscribeAll(handler: (event: GameEvent) => void): Unsubscribe
}

// Example: Timer system subscribes to game events
eventBus.subscribe('QUESTION_DISPLAYED', () => timer.resume())
eventBus.subscribe('ANSWER_LOCKED', () => timer.pause())
eventBus.subscribe('TIMER_MODIFIED', (event) => timer.adjust(event.delta_seconds))
```

### 2. State Machine

Game state is explicit and transitions are controlled.

```typescript
type GameState =
  | { phase: 'IDLE' }
  | { phase: 'QUESTION'; question: Question; cards_active: Card[]; votes: VoteState }
  | { phase: 'ANSWER_REVEAL'; question: Question; correct: boolean; explanation: string }
  | { phase: 'GAME_OVER'; won: boolean; score: number; surplus_points?: number }
  | { phase: 'STORE'; available_cards: Card[]; currency: number } // Campaign only
  | { phase: 'CAMPAIGN_MAP'; current_level: number } // Campaign only

interface GameContext {
  state: GameState
  score: number
  timer: TimerState
  question_index: number
  total_questions: number
  passing_grade: number
  card_deck: Card[]
  active_cards: Card[] // Permanent cards on table
  mode: 'normal' | 'campaign'
}

type GameAction =
  | { type: 'START_GAME'; quiz_id: string }
  | { type: 'NEXT_QUESTION' }
  | { type: 'SUBMIT_ANSWER'; answer: string }
  | { type: 'VOTE_CARD'; card_id: string; user: string }
  | { type: 'REVEAL_ANSWER' }
  | { type: 'END_GAME' }

function transition(context: GameContext, action: GameAction): GameContext {
  // State machine logic
}
```

### 3. Timer System

#### Requirements
- Pause/resume capability
- Modification via delta (±seconds)
- Expiration detection
- No drift (accurate timing)

#### Implementation Approach

```typescript
interface TimerState {
  running: boolean
  remaining_ms: number
  last_update_timestamp?: number
}

class Timer {
  private state: TimerState
  private interval?: NodeJS.Timeout
  private eventBus: EventBus

  start(duration_ms: number): void {
    this.state = {
      running: true,
      remaining_ms: duration_ms,
      last_update_timestamp: Date.now()
    }
    this.startTicking()
    this.eventBus.publish({ type: 'TIMER_STARTED', duration_seconds: duration_ms / 1000 })
  }

  pause(): void {
    if (!this.state.running) return

    this.updateRemaining()
    this.state.running = false
    this.stopTicking()
    this.eventBus.publish({
      type: 'TIMER_PAUSED',
      remaining_seconds: this.state.remaining_ms / 1000
    })
  }

  resume(): void {
    if (this.state.running) return

    this.state.running = true
    this.state.last_update_timestamp = Date.now()
    this.startTicking()
    this.eventBus.publish({
      type: 'TIMER_RESUMED',
      remaining_seconds: this.state.remaining_ms / 1000
    })
  }

  adjust(delta_seconds: number): void {
    this.updateRemaining()
    this.state.remaining_ms += delta_seconds * 1000
    this.eventBus.publish({ type: 'TIMER_MODIFIED', delta_seconds })
  }

  private updateRemaining(): void {
    if (!this.state.running || !this.state.last_update_timestamp) return

    const now = Date.now()
    const elapsed = now - this.state.last_update_timestamp
    this.state.remaining_ms = Math.max(0, this.state.remaining_ms - elapsed)
    this.state.last_update_timestamp = now
  }

  private startTicking(): void {
    this.interval = setInterval(() => {
      this.updateRemaining()

      if (this.state.remaining_ms <= 0) {
        this.stopTicking()
        this.eventBus.publish({ type: 'TIMER_EXPIRED' })
      }
    }, 100) // Update every 100ms for smooth display
  }

  private stopTicking(): void {
    if (this.interval) {
      clearInterval(this.interval)
      this.interval = undefined
    }
  }

  getRemaining(): number {
    this.updateRemaining()
    return this.state.remaining_ms
  }
}
```

### 4. Card Effect System

Cards modify game state through composable effects.

#### Effect Types

```typescript
type CardEffect =
  | { type: 'ELIMINATE_WRONG_ANSWER'; count: number }
  | { type: 'ADD_TIME'; seconds: number }
  | { type: 'SUBTRACT_TIME'; seconds: number }
  | { type: 'ADD_POINTS'; points: number }
  | { type: 'MULTIPLY_POINTS'; multiplier: number }
  | { type: 'SET_POINTS'; points: number }
  | { type: 'BAN_CARD_TYPE'; card_type: CardType }
  | { type: 'REMOVE_CARD'; card_id: string }
  | { type: 'ADD_CARD_SLOT' }
  | { type: 'MODIFY_VOTE_REQUIREMENT'; delta: number }
  | { type: 'PLAY_EXTRA_CARD' } // Allows deploying another card

interface Card {
  id: string
  name: string
  type: 'resistance' | 'palestinian' | 'politics' | 'idf' | 'event'
  permanence: 'permanent' | 'temporary'
  effects: CardEffect[]
  cost?: number // For store purchases
  vote_requirement: number
  triggers: CardTrigger[]
  counters?: string[] // Card IDs this card counters
  countered_by?: string[] // Card IDs that counter this card
  visual?: {
    image: string
    sound?: string
    animation?: string
  }
}

type CardTrigger =
  | { when: 'ON_DEPLOY' }
  | { when: 'ON_QUESTION_START' }
  | { when: 'ON_ANSWER_SUBMITTED' }
  | { when: 'ON_ANSWER_CORRECT' }
  | { when: 'CONDITIONAL'; condition: string } // e.g., "timer > 50%"
```

#### Effect Application

```typescript
class CardEffectResolver {
  applyEffects(effects: CardEffect[], context: GameContext): GameContext {
    let newContext = context

    for (const effect of effects) {
      newContext = this.applyEffect(effect, newContext)
    }

    return newContext
  }

  private applyEffect(effect: CardEffect, context: GameContext): GameContext {
    switch (effect.type) {
      case 'ELIMINATE_WRONG_ANSWER':
        return this.eliminateWrongAnswers(context, effect.count)

      case 'ADD_TIME':
        this.eventBus.publish({ type: 'TIMER_MODIFIED', delta_seconds: effect.seconds })
        return context

      case 'ADD_POINTS':
        if (context.state.phase === 'QUESTION') {
          const question = context.state.question
          return {
            ...context,
            state: {
              ...context.state,
              question: {
                ...question,
                points: question.points + effect.points
              }
            }
          }
        }
        return context

      // ... other effects
    }
  }

  private eliminateWrongAnswers(context: GameContext, count: number): GameContext {
    if (context.state.phase !== 'QUESTION') return context

    const question = context.state.question
    const wrongAnswers = question.options.filter(opt => !opt.correct)
    const toEliminate = wrongAnswers.slice(0, count)

    return {
      ...context,
      state: {
        ...context.state,
        question: {
          ...question,
          options: question.options.filter(opt => !toEliminate.includes(opt))
        }
      }
    }
  }
}
```

### 5. Vote System

Track votes for card deployment.

```typescript
interface VoteState {
  [card_id: string]: {
    voters: Set<string>
    required: number
  }
}

class VoteManager {
  private votes: VoteState = {}
  private eventBus: EventBus

  vote(user: string, card_id: string, card: Card): boolean {
    if (!this.votes[card_id]) {
      this.votes[card_id] = {
        voters: new Set(),
        required: card.vote_requirement
      }
    }

    const voteState = this.votes[card_id]

    // Prevent duplicate votes
    if (voteState.voters.has(user)) {
      return false
    }

    voteState.voters.add(user)

    this.eventBus.publish({
      type: 'CARD_VOTE_CAST',
      player: user,
      card_id
    })

    // Check if threshold reached
    if (voteState.voters.size >= voteState.required) {
      this.eventBus.publish({
        type: 'CARD_DEPLOYED',
        card
      })
      return true
    }

    return false
  }

  reset(): void {
    this.votes = {}
  }

  getVoteCount(card_id: string): { current: number; required: number } {
    const voteState = this.votes[card_id]
    if (!voteState) {
      return { current: 0, required: 0 }
    }
    return {
      current: voteState.voters.size,
      required: voteState.required
    }
  }
}
```

### 6. Answer Processing

Handle answer submission and consensus.

```typescript
interface AnswerState {
  submissions: Map<string, string> // user -> answer
  locked: boolean
  correct_answer?: string
}

class AnswerManager {
  private state: AnswerState = {
    submissions: new Map(),
    locked: false
  }
  private eventBus: EventBus

  submit(user: string, answer: string): void {
    if (this.state.locked) return

    // Normalize answer
    answer = answer.toUpperCase().trim()
    if (!['A', 'B', 'C', 'D'].includes(answer)) return

    this.state.submissions.set(user, answer)
    this.eventBus.publish({
      type: 'ANSWER_SUBMITTED',
      player: user,
      answer
    })

    // Check for consensus (2 matching answers)
    this.checkConsensus()
  }

  private checkConsensus(): void {
    const answerCounts = new Map<string, number>()

    for (const answer of this.state.submissions.values()) {
      answerCounts.set(answer, (answerCounts.get(answer) || 0) + 1)
    }

    for (const [answer, count] of answerCounts.entries()) {
      if (count >= 2) {
        this.lockAnswer(answer)
        return
      }
    }
  }

  private lockAnswer(answer: string): void {
    this.state.locked = true
    this.state.correct_answer = answer
    this.eventBus.publish({
      type: 'ANSWER_LOCKED',
      answer
    })
  }

  reset(): void {
    this.state = {
      submissions: new Map(),
      locked: false
    }
  }

  isLocked(): boolean {
    return this.state.locked
  }

  getLockedAnswer(): string | undefined {
    return this.state.correct_answer
  }
}
```

---

## Twitch Integration

### Chat Connection

Use Twitch IRC or official API.

#### Option 1: Twitch IRC (Simple)

```typescript
import tmi from 'tmi.js'

class TwitchChatClient {
  private client: tmi.Client
  private eventBus: EventBus

  connect(channel: string, oauth_token: string): void {
    this.client = new tmi.Client({
      options: { debug: false },
      connection: {
        reconnect: true,
        secure: true
      },
      identity: {
        username: 'your_bot_name',
        password: oauth_token
      },
      channels: [channel]
    })

    this.client.on('message', (channel, tags, message, self) => {
      if (self) return // Ignore bot's own messages

      this.eventBus.publish({
        type: 'CHAT_MESSAGE',
        user: tags.username!,
        message: message.trim()
      })
    })

    this.client.connect()
  }

  disconnect(): void {
    this.client.disconnect()
  }
}
```

#### Option 2: Twitch EventSub (Advanced)

For production, use EventSub for more reliable, scalable chat handling.

### Command Parsing

```typescript
interface Command {
  type: 'ANSWER' | 'VOTE_CARD' | 'UNKNOWN'
  payload?: any
}

class CommandParser {
  parse(message: string): Command {
    const trimmed = message.trim().toLowerCase()

    // Check for simple answer (A, B, C, D)
    if (/^[abcd]$/.test(trimmed)) {
      return {
        type: 'ANSWER',
        payload: { answer: trimmed.toUpperCase() }
      }
    }

    // Check for card vote command
    const voteMatch = trimmed.match(/^(?:\/)?use\s+(.+)$/)
    if (voteMatch) {
      const cardName = voteMatch[1]
      return {
        type: 'VOTE_CARD',
        payload: { card_name: cardName }
      }
    }

    return { type: 'UNKNOWN' }
  }

  // Fuzzy match card name to card ID
  matchCardName(input: string, availableCards: Card[]): Card | undefined {
    const normalized = input.toLowerCase().replace(/[^a-z0-9]/g, '')

    return availableCards.find(card => {
      const cardName = card.name.toLowerCase().replace(/[^a-z0-9]/g, '')
      return cardName.includes(normalized) || normalized.includes(cardName)
    })
  }
}
```

### Integration with Game Loop

```typescript
// Wire chat events to game logic
eventBus.subscribe('CHAT_MESSAGE', (event) => {
  const command = commandParser.parse(event.message)

  switch (command.type) {
    case 'ANSWER':
      answerManager.submit(event.user, command.payload.answer)
      break

    case 'VOTE_CARD':
      const card = commandParser.matchCardName(
        command.payload.card_name,
        gameContext.active_cards
      )
      if (card) {
        const deployed = voteManager.vote(event.user, card.id, card)
        if (deployed) {
          cardEffectResolver.applyEffects(card.effects, gameContext)
        }
      }
      break
  }
})
```

---

## Data Management

### Quiz Loading

```typescript
interface QuizData {
  meta: {
    id: string
    title: string
    description: string
    passing_grade: number
    initial_timer_minutes: number
  }
  questions: Question[]
  cards: Card[]
}

class QuizLoader {
  async load(quiz_id: string): Promise<QuizData> {
    // Load from file, API, or database
    const raw = await readFile(`quizzes/${quiz_id}.json`, 'utf-8')
    const data = JSON.parse(raw)

    // Validate schema
    this.validate(data)

    return data
  }

  private validate(data: any): void {
    // Use JSON Schema or Zod for validation
    // Ensure all required fields present
    // Check question format, card effects, etc.
  }
}
```

### Randomization

```typescript
class Randomizer {
  private rng: () => number

  constructor(seed?: number) {
    // Use seeded RNG for reproducibility in tests
    this.rng = seed ? this.seededRandom(seed) : Math.random
  }

  shuffleQuestions(questions: Question[]): Question[] {
    return this.shuffle([...questions])
  }

  shuffleOptions(question: Question): Question {
    return {
      ...question,
      options: this.shuffle([...question.options])
    }
  }

  drawCard(deck: Card[], bannedTypes: Set<string> = new Set()): Card | undefined {
    const available = deck.filter(card => !bannedTypes.has(card.type))
    if (available.length === 0) return undefined

    const index = Math.floor(this.rng() * available.length)
    return available[index]
  }

  private shuffle<T>(array: T[]): T[] {
    for (let i = array.length - 1; i > 0; i--) {
      const j = Math.floor(this.rng() * (i + 1))
      ;[array[i], array[j]] = [array[j], array[i]]
    }
    return array
  }

  private seededRandom(seed: number): () => number {
    let state = seed
    return () => {
      state = (state * 9301 + 49297) % 233280
      return state / 233280
    }
  }
}
```

---

## Rendering / UI

### Web-Based UI (Recommended)

Use HTML/CSS/JS for OBS browser source compatibility.

#### Technology Options

1. **Vanilla JS + Canvas**: Maximum control, good performance
2. **React**: Component-based, easier state management
3. **Svelte**: Minimal bundle size, reactive
4. **Vue**: Good balance of simplicity and power

#### Communication: Backend ↔ Frontend

**Option A: WebSocket**
```typescript
// Backend sends state updates
ws.send(JSON.stringify({
  type: 'STATE_UPDATE',
  state: gameContext.state,
  timer: timer.getRemaining(),
  score: gameContext.score
}))

// Frontend sends user commands (if not using Twitch chat)
ws.send(JSON.stringify({
  type: 'SUBMIT_ANSWER',
  answer: 'B'
}))
```

**Option B: Server-Sent Events (SSE)**
One-way communication for state updates (simpler if input only via Twitch).

#### UI Component Structure

```
App
├── GameHeader
│   ├── Timer
│   └── Score
├── QuestionDisplay
│   ├── QuestionText
│   └── OptionsList
│       └── Option (x4)
├── CardTray
│   └── CardSlot (x4)
│       ├── Card
│       └── VoteIndicator
└── AnswerReveal (conditional)
    ├── Explanation
    └── ScoreAnimation
```

### Styling Requirements

- **Responsive**: Scale to different resolutions
- **Readable**: High contrast, large text
- **Accessible**: Screen reader support, keyboard navigation
- **Themed**: Consistent color scheme, Palestinian flag colors?
- **Animated**: Smooth transitions, card effects, timer warnings

---

## Persistence Layer

### Storage Backend Architecture

**Key Requirement**: Users must be able to resume progress across quiz sessions.

**Design Pattern**: Trait/interface-based abstraction for swappable storage backends

```
┌────────────────────────────────────────┐
│      Campaign Manager / Services       │
└──────────────────┬─────────────────────┘
                   │
                   ▼
┌────────────────────────────────────────┐
│      StorageBackend Trait/Interface    │
│                                        │
│  ✅ Generic, reusable interface        │
│  ✅ Framework code, not game code      │
└──────────────────┬─────────────────────┘
                   │
        ┌──────────┴────────┬────────────┐
        │                   │            │
┌───────▼─────┐  ┌─────────▼──┐  ┌──────▼────┐
│  Supabase   │  │ PostgreSQL │  │  SQLite   │
│     Impl    │  │    Impl    │  │   Impl    │
└─────────────┘  └────────────┘  └───────────┘
```

### Storage Backend Trait

**Rust Interface**:
```rust
#[async_trait]
pub trait StorageBackend: Send + Sync {
    // User management
    async fn get_or_create_user(&self, twitch_id: &str, username: &str) -> Result<User>;
    async fn update_last_seen(&self, user_id: &str) -> Result<()>;

    // Card ownership - CRITICAL for campaign mode
    async fn get_user_cards(&self, user_id: &str, quiz_id: &str) -> Result<Vec<UserCard>>;
    async fn add_card_to_user(
        &self,
        user_id: &str,
        card_id: &str,
        quiz_id: &str,
        method: AcquisitionMethod
    ) -> Result<UserCard>;
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
    async fn get_store_deposits(&self, user_id: &str, quiz_id: &str) -> Result<HashMap<CardType, i32>>;
    async fn deposit_card_for_upgrade(
        &self,
        user_id: &str,
        card_id: &str,
        quiz_id: &str,
        card_type: CardType
    ) -> Result<()>;
    async fn upgrade_store(&self, user_id: &str, quiz_id: &str) -> Result<i32>;

    // Statistics
    async fn record_game_result(
        &self,
        user_id: &str,
        quiz_id: &str,
        result: GameResult
    ) -> Result<()>;
}

pub struct GameResult {
    pub won: bool,
    pub score: i32,
    pub questions_answered: i32,
    pub correct_answers: i32,
}
```

**TypeScript Interface**:
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
  userOwnsCard(userId: string, cardId: string, quizId: string): Promise<boolean>;

  // Campaign progress
  getCampaignProgress(userId: string, quizId: string): Promise<CampaignProgress>;
  updateCampaignProgress(progress: CampaignProgress): Promise<void>;
  completeLevel(userId: string, quizId: string, level: number, surplusPoints: number): Promise<void>;

  // Store management
  getStoreDeposits(userId: string, quizId: string): Promise<Map<CardType, number>>;
  depositCardForUpgrade(
    userId: string,
    cardId: string,
    quizId: string,
    cardType: CardType
  ): Promise<void>;
  upgradeStore(userId: string, quizId: string): Promise<number>; // Returns new store level

  // Statistics
  recordGameResult(userId: string, quizId: string, result: GameResult): Promise<void>;
}
```

### Supabase Implementation

**Initial Backend**: Supabase (PostgreSQL-based, free tier, real-time capabilities)

#### Database Schema

```sql
-- Users table
CREATE TABLE users (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  twitch_id VARCHAR(255) UNIQUE NOT NULL,
  twitch_username VARCHAR(255) NOT NULL,
  display_name VARCHAR(255),
  created_at TIMESTAMP DEFAULT NOW(),
  last_seen TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_users_twitch_id ON users(twitch_id);

-- Card ownership table
CREATE TABLE user_cards (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  user_id UUID REFERENCES users(id) ON DELETE CASCADE,
  card_id VARCHAR(255) NOT NULL,
  quiz_id VARCHAR(255) NOT NULL,
  acquisition_method VARCHAR(50) NOT NULL CHECK (acquisition_method IN ('earned', 'purchased', 'deposited')),
  acquisition_date TIMESTAMP DEFAULT NOW(),
  is_deposited BOOLEAN DEFAULT FALSE,
  UNIQUE(user_id, card_id, quiz_id)
);

CREATE INDEX idx_user_cards_user_id ON user_cards(user_id);
CREATE INDEX idx_user_cards_user_quiz ON user_cards(user_id, quiz_id);

-- Campaign progress table
CREATE TABLE campaign_progress (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  user_id UUID REFERENCES users(id) ON DELETE CASCADE,
  quiz_id VARCHAR(255) NOT NULL,
  current_level INTEGER DEFAULT 1,
  store_level INTEGER DEFAULT 1,
  currency INTEGER DEFAULT 0,
  completed_levels INTEGER[] DEFAULT '{}',
  updated_at TIMESTAMP DEFAULT NOW(),
  UNIQUE(user_id, quiz_id)
);

CREATE INDEX idx_campaign_user_quiz ON campaign_progress(user_id, quiz_id);

-- Store deposits table
CREATE TABLE store_deposits (
  user_id UUID REFERENCES users(id) ON DELETE CASCADE,
  quiz_id VARCHAR(255) NOT NULL,
  card_type VARCHAR(50) NOT NULL,
  count INTEGER DEFAULT 0,
  updated_at TIMESTAMP DEFAULT NOW(),
  PRIMARY KEY(user_id, quiz_id, card_type)
);

-- Quiz statistics table
CREATE TABLE quiz_stats (
  user_id UUID REFERENCES users(id) ON DELETE CASCADE,
  quiz_id VARCHAR(255) NOT NULL,
  total_games INTEGER DEFAULT 0,
  wins INTEGER DEFAULT 0,
  losses INTEGER DEFAULT 0,
  total_questions_answered INTEGER DEFAULT 0,
  total_correct_answers INTEGER DEFAULT 0,
  best_score INTEGER DEFAULT 0,
  last_played TIMESTAMP,
  PRIMARY KEY(user_id, quiz_id)
);
```

#### Supabase Client Implementation

```typescript
import { createClient, SupabaseClient } from '@supabase/supabase-js'

class SupabaseStorageBackend implements StorageBackend {
  private client: SupabaseClient

  constructor(url: string, anonKey: string) {
    this.client = createClient(url, anonKey)
  }

  async getOrCreateUser(twitchId: string, username: string): Promise<User> {
    // Try to get existing user
    const { data: existing } = await this.client
      .from('users')
      .select('*')
      .eq('twitch_id', twitchId)
      .single()

    if (existing) {
      // Update last_seen
      await this.client
        .from('users')
        .update({ last_seen: new Date(), twitch_username: username })
        .eq('id', existing.id)

      return existing
    }

    // Create new user
    const { data: newUser, error } = await this.client
      .from('users')
      .insert({
        twitch_id: twitchId,
        twitch_username: username,
        display_name: username
      })
      .select()
      .single()

    if (error) throw error
    return newUser
  }

  async getUserCards(userId: string, quizId: string): Promise<UserCard[]> {
    const { data, error } = await this.client
      .from('user_cards')
      .select('*')
      .eq('user_id', userId)
      .eq('quiz_id', quizId)
      .eq('is_deposited', false) // Only non-deposited cards are "owned"

    if (error) throw error
    return data || []
  }

  async addCardToUser(
    userId: string,
    cardId: string,
    quizId: string,
    method: AcquisitionMethod
  ): Promise<UserCard> {
    const { data, error } = await this.client
      .from('user_cards')
      .insert({
        user_id: userId,
        card_id: cardId,
        quiz_id: quizId,
        acquisition_method: method
      })
      .select()
      .single()

    if (error) {
      // Handle unique constraint violation (user already owns card)
      if (error.code === '23505') {
        // Return existing card
        const { data: existing } = await this.client
          .from('user_cards')
          .select('*')
          .eq('user_id', userId)
          .eq('card_id', cardId)
          .eq('quiz_id', quizId)
          .single()

        return existing!
      }
      throw error
    }

    return data
  }

  async getCampaignProgress(userId: string, quizId: string): Promise<CampaignProgress> {
    const { data, error } = await this.client
      .from('campaign_progress')
      .select('*')
      .eq('user_id', userId)
      .eq('quiz_id', quizId)
      .single()

    if (error && error.code === 'PGRST116') {
      // No progress exists, create initial progress
      const { data: newProgress, error: insertError } = await this.client
        .from('campaign_progress')
        .insert({
          user_id: userId,
          quiz_id: quizId,
          current_level: 1,
          store_level: 1,
          currency: 0,
          completed_levels: []
        })
        .select()
        .single()

      if (insertError) throw insertError
      return newProgress!
    }

    if (error) throw error
    return data!
  }

  async completeLevel(
    userId: string,
    quizId: string,
    level: number,
    surplusPoints: number
  ): Promise<void> {
    const progress = await this.getCampaignProgress(userId, quizId)

    const { error } = await this.client
      .from('campaign_progress')
      .update({
        current_level: Math.max(progress.current_level, level + 1),
        completed_levels: [...new Set([...progress.completed_levels, level])],
        currency: progress.currency + surplusPoints,
        updated_at: new Date()
      })
      .eq('user_id', userId)
      .eq('quiz_id', quizId)

    if (error) throw error
  }

  async depositCardForUpgrade(
    userId: string,
    cardId: string,
    quizId: string,
    cardType: CardType
  ): Promise<void> {
    // Mark card as deposited
    await this.client
      .from('user_cards')
      .update({ is_deposited: true })
      .eq('user_id', userId)
      .eq('card_id', cardId)
      .eq('quiz_id', quizId)

    // Increment store deposit count
    const { data: existing } = await this.client
      .from('store_deposits')
      .select('count')
      .eq('user_id', userId)
      .eq('quiz_id', quizId)
      .eq('card_type', cardType)
      .single()

    if (existing) {
      await this.client
        .from('store_deposits')
        .update({ count: existing.count + 1, updated_at: new Date() })
        .eq('user_id', userId)
        .eq('quiz_id', quizId)
        .eq('card_type', cardType)
    } else {
      await this.client
        .from('store_deposits')
        .insert({
          user_id: userId,
          quiz_id: quizId,
          card_type: cardType,
          count: 1
        })
    }
  }

  async upgradeStore(userId: string, quizId: string): Promise<number> {
    const progress = await this.getCampaignProgress(userId, quizId)
    const newLevel = progress.store_level + 1

    const { error } = await this.client
      .from('campaign_progress')
      .update({ store_level: newLevel, updated_at: new Date() })
      .eq('user_id', userId)
      .eq('quiz_id', quizId)

    if (error) throw error
    return newLevel
  }

  async recordGameResult(
    userId: string,
    quizId: string,
    result: GameResult
  ): Promise<void> {
    const { data: existing } = await this.client
      .from('quiz_stats')
      .select('*')
      .eq('user_id', userId)
      .eq('quiz_id', quizId)
      .single()

    if (existing) {
      await this.client
        .from('quiz_stats')
        .update({
          total_games: existing.total_games + 1,
          wins: existing.wins + (result.won ? 1 : 0),
          losses: existing.losses + (result.won ? 0 : 1),
          total_questions_answered: existing.total_questions_answered + result.questions_answered,
          total_correct_answers: existing.total_correct_answers + result.correct_answers,
          best_score: Math.max(existing.best_score, result.score),
          last_played: new Date()
        })
        .eq('user_id', userId)
        .eq('quiz_id', quizId)
    } else {
      await this.client
        .from('quiz_stats')
        .insert({
          user_id: userId,
          quiz_id: quizId,
          total_games: 1,
          wins: result.won ? 1 : 0,
          losses: result.won ? 0 : 1,
          total_questions_answered: result.questions_answered,
          total_correct_answers: result.correct_answers,
          best_score: result.score,
          last_played: new Date()
        })
    }
  }
}
```

### User Identification via Twitch

**Automatic User Creation**:
- Extract Twitch user ID from IRC tags in chat messages
- Call `getOrCreateUser()` on first interaction
- No manual login/signup required

```typescript
// Twitch IRC message parsing
class TwitchChatClient {
  private storage: StorageBackend

  async handleMessage(tags: Record<string, string>, message: string) {
    const twitchId = tags['user-id']
    const username = tags['display-name'] || tags['username']

    // Get or create user automatically
    const user = await this.storage.getOrCreateUser(twitchId, username)

    // Process command with user context
    this.processCommand(user, message)
  }
}
```

### Configuration

```yaml
# config.yml
storage:
  backend: supabase # or 'postgresql', 'sqlite'

  supabase:
    url: ${SUPABASE_URL}
    anon_key: ${SUPABASE_ANON_KEY}
    # Optional: service_role_key for admin operations

  postgresql:
    connection_string: ${DATABASE_URL}

  sqlite:
    file_path: ./data/cgq.db
```

### Migration Between Backends

**Export/Import Tools**:
```bash
# Export from Supabase
cgq-cli export --backend supabase --output backup.json

# Import to PostgreSQL
cgq-cli import --backend postgresql --input backup.json

# Migrate directly
cgq-cli migrate --from supabase --to postgresql
```

---

## Campaign Mode Specifics

### Campaign Manager with Persistence

```typescript
class CampaignManager {
  private storage: StorageBackend
  private userId: string
  private quizId: string

  constructor(storage: StorageBackend, userId: string, quizId: string) {
    this.storage = storage
    this.userId = userId
    this.quizId = quizId
  }

  async loadProgress(): Promise<CampaignProgress> {
    return await this.storage.getCampaignProgress(this.userId, this.quizId)
  }

  async completeLevel(level: number, finalScore: number, passingGrade: number): Promise<void> {
    const surplusPoints = Math.max(0, finalScore - passingGrade)
    await this.storage.completeLevel(this.userId, this.quizId, level, surplusPoints)
  }

  async purchaseCard(card: Card): Promise<boolean> {
    const progress = await this.loadProgress()

    if (progress.currency < card.cost!) {
      return false
    }

    // Deduct currency
    progress.currency -= card.cost!
    await this.storage.updateCampaignProgress(progress)

    // Add card to user's collection
    await this.storage.addCardToUser(
      this.userId,
      card.id,
      this.quizId,
      AcquisitionMethod.Purchased
    )

    return true
  }

  async depositCardForUpgrade(cardId: string, cardType: CardType): Promise<boolean> {
    // Check if user owns the card
    const owns = await this.storage.userOwnsCard(this.userId, cardId, this.quizId)
    if (!owns) return false

    // Deposit card
    await this.storage.depositCardForUpgrade(this.userId, cardId, this.quizId, cardType)

    // Check if store can be upgraded
    const deposits = await this.storage.getStoreDeposits(this.userId, this.quizId)
    const storeConfig = await this.getStoreConfig()

    if (deposits.get(cardType) >= storeConfig.upgrade_requirement.count) {
      await this.storage.upgradeStore(this.userId, this.quizId)
      return true
    }

    return false
  }

  async getOwnedCards(): Promise<UserCard[]> {
    return await this.storage.getUserCards(this.userId, this.quizId)
  }
```

### Store Implementation

```typescript
interface StoreConfig {
  level: number
  slots: number
  prices: {
    resistance: number
    palestinian: number
    politics: number
  }
  upgrade_requirement: {
    type: string
    count: number
  }
}

class Store {
  generateOffering(
    all_cards: Card[],
    unlocked_cards: Set<string>,
    slots: number
  ): Card[] {
    // Filter to cards not yet unlocked
    const available = all_cards.filter(c => !unlocked_cards.has(c.id))

    // Randomly select {slots} cards
    return this.randomizer.shuffle(available).slice(0, slots)
  }

  getPrice(card: Card, store_level: number): number {
    const config = this.getStoreConfig(store_level)
    const basePrice = config.prices[card.type]

    // Apply any special pricing rules
    return basePrice
  }
}
```

---

## Testing Strategy

### Unit Tests

Test individual systems in isolation.

```typescript
describe('Timer', () => {
  it('should pause and resume correctly', () => {
    const timer = new Timer(eventBus)
    timer.start(60000) // 1 minute

    // Wait 100ms
    await sleep(100)
    timer.pause()

    const remaining1 = timer.getRemaining()
    expect(remaining1).toBeLessThan(60000)
    expect(remaining1).toBeGreaterThan(59800)

    // Wait another 100ms
    await sleep(100)
    const remaining2 = timer.getRemaining()

    // Should not have decreased while paused
    expect(remaining2).toBe(remaining1)

    timer.resume()
    await sleep(100)
    const remaining3 = timer.getRemaining()

    // Should have decreased after resume
    expect(remaining3).toBeLessThan(remaining2)
  })

  it('should apply time modifications', () => {
    const timer = new Timer(eventBus)
    timer.start(60000)

    timer.adjust(30) // Add 30 seconds
    expect(timer.getRemaining()).toBeGreaterThan(89000)

    timer.adjust(-45) // Subtract 45 seconds
    expect(timer.getRemaining()).toBeLessThan(46000)
  })
})

describe('CardEffectResolver', () => {
  it('should eliminate wrong answers', () => {
    const question: Question = {
      id: 'q1',
      text: 'Test?',
      options: [
        { id: 'a', text: 'Wrong 1', correct: false },
        { id: 'b', text: 'Correct', correct: true },
        { id: 'c', text: 'Wrong 2', correct: false },
        { id: 'd', text: 'Wrong 3', correct: false }
      ],
      points: 2
    }

    const context: GameContext = {
      state: { phase: 'QUESTION', question, cards_active: [], votes: {} },
      // ... other fields
    }

    const effect: CardEffect = { type: 'ELIMINATE_WRONG_ANSWER', count: 2 }
    const newContext = resolver.applyEffect(effect, context)

    expect(newContext.state.question.options.length).toBe(2)
    expect(newContext.state.question.options.some(o => o.correct)).toBe(true)
  })
})
```

### Integration Tests

Test component interactions.

```typescript
describe('Game Flow', () => {
  it('should complete a full question cycle', async () => {
    const game = new GameController(quizData, eventBus)
    game.start()

    // Should display first question
    expect(game.getState().phase).toBe('QUESTION')

    // Submit answers
    answerManager.submit('user1', 'A')
    answerManager.submit('user2', 'B')
    answerManager.submit('user3', 'B')

    // Should lock on consensus
    await waitFor(() => answerManager.isLocked())
    expect(answerManager.getLockedAnswer()).toBe('B')

    // Should transition to answer reveal
    await waitFor(() => game.getState().phase === 'ANSWER_REVEAL')

    // Should auto-advance to next question
    await waitFor(() => game.getState().phase === 'QUESTION')
    expect(game.getQuestionIndex()).toBe(1)
  })
})
```

### End-to-End Tests

Test full user flows with simulated Twitch chat.

```typescript
describe('E2E: Complete Quiz', () => {
  it('should handle a full quiz playthrough', async () => {
    const chatSimulator = new TwitchChatSimulator()
    const game = new GameController(quizData, eventBus)

    game.start()

    // Simulate chat answering questions
    for (let i = 0; i < 15; i++) {
      await waitForState('QUESTION')

      // Users submit answers
      chatSimulator.send('user1', 'A')
      chatSimulator.send('user2', 'B')
      chatSimulator.send('user3', 'B')

      await waitForState('ANSWER_REVEAL')
    }

    await waitForState('GAME_OVER')

    const finalState = game.getState()
    expect(finalState.won).toBeDefined()
    expect(finalState.score).toBeGreaterThan(0)
  })
})
```

---

## Deployment

### Build Process: Content + Engine → Game

**CRITICAL: The build process compiles content data WITH the engine to produce a playable game.**

#### Build Workflow

```
┌──────────────────┐
│  Content Files   │
│  - questions.yml │
│  - cards.yml     │
│  - config.yml    │
│  - assets/*      │
└────────┬─────────┘
         │
         ▼
    ┌─────────┐
    │Validator│ ← Schema validation
    │ /Linter │
    └────┬────┘
         │ Valid?
         ▼
┌────────────────┐       ┌─────────────┐
│ Content Bundle │   +   │ CGQ Engine  │
│   (validated)  │       │  (binary)   │
└────────┬───────┘       └──────┬──────┘
         │                      │
         └──────────┬───────────┘
                    │ Compile
                    ▼
         ┌──────────────────────┐
         │   Game Binary        │
         │ (Engine + Content)   │
         │  - cgq-palestine     │
         │  - cgq-sciencequiz   │
         │  - cgq-history       │
         └──────────────────────┘
```

#### Example Build Commands

```bash
# 1. Validate content
cgq-cli validate ./content/palestinian-quiz/

# 2. Build game binary
cgq-cli build \
  --content ./content/palestinian-quiz/ \
  --output ./dist/cgq-palestinian-quiz

# 3. Result: Single executable with embedded content
./dist/cgq-palestinian-quiz --twitch-channel kairos_rev

# Alternative: Runtime content loading (development)
cgq-engine --content-dir ./content/palestinian-quiz/
```

#### Content Directory Structure

```
content/palestinian-quiz/
├── quiz.yml              # Quiz metadata
├── questions/
│   ├── part1.yml         # Questions
│   ├── part2.yml
│   └── part3.yml
├── cards/
│   ├── resistance.yml    # Card definitions
│   ├── palestinian.yml
│   └── politics.yml
├── campaign/
│   └── levels.yml        # Campaign configuration
├── config.yml            # Game rules (timer, scoring, etc.)
└── assets/
    ├── cards/            # Card artwork
    ├── maps/             # Campaign maps
    └── sounds/           # Audio effects
```

### Architecture Options

**Option 1: Single Binary + Web UI** (Recommended for MVP)
- Rust/Go backend serving HTTP + WebSocket
- Static HTML/CSS/JS frontend
- Content embedded at build time OR loaded from data directory
- Runs on streamer's machine
- OBS points to `localhost:3000`

**Benefits**:
- Simplest deployment
- No hosting costs
- Content creators can distribute binaries with their quiz content
- Easier development iteration

**Option 2: Cloud Hosted**
- Backend on cloud (AWS, GCP, Heroku)
- Frontend served as static site
- Multiple streamers can use same instance
- Content uploaded via web interface
- Requires user authentication, multi-tenancy

**Benefits**:
- No installation for streamers
- Centralized content repository
- Easy updates

**Option 3: Twitch Extension**
- Full integration with Twitch platform
- Appears as overlay on stream
- More complex approval process
- Best user experience for viewers

**Benefits**:
- Native Twitch integration
- Professional appearance
- Discoverable by other streamers

### Recommendation: Hybrid Approach

**Phase 1**: Option 1 (local binary) for development and early adopters
**Phase 2**: Option 2 (cloud) for broader distribution
**Phase 3**: Option 3 (Twitch extension) for maximum reach

### Build Tool Design

```bash
# CGQ CLI - Content build tool

# Create new quiz from template
cgq-cli new my-quiz

# Validate content
cgq-cli validate ./my-quiz/

# Build standalone game
cgq-cli build ./my-quiz/ --output ./games/my-quiz

# Run in dev mode (hot reload)
cgq-cli dev ./my-quiz/

# Package for distribution
cgq-cli package ./my-quiz/ --format [binary|zip|docker]
```

### Configuration

```yaml
# config.yaml
quiz:
  data_dir: ./quizzes

twitch:
  channel: kairos_rev
  oauth_token: ${TWITCH_OAUTH_TOKEN}

server:
  host: 127.0.0.1
  port: 3000

campaign:
  save_dir: ./saves
```

---

## Performance Considerations

### Optimization Targets

- **Chat processing latency**: <100ms
- **UI update rate**: 60 FPS
- **Timer accuracy**: ±100ms
- **Memory usage**: <100MB
- **CPU usage**: <5% (idle), <20% (active)

### Strategies

1. **Event batching**: Group rapid events before UI update
2. **Lazy rendering**: Only re-render changed components
3. **Connection pooling**: Reuse Twitch connections
4. **Efficient data structures**: Use appropriate collections (Map vs Object)
5. **Debouncing**: Limit chat spam impact

---

## Security Considerations

### Input Validation

- Sanitize all chat input
- Validate quiz/card JSON schemas
- Rate limit command processing
- Prevent injection attacks

### Authentication

- Secure Twitch OAuth token storage
- Don't expose tokens in client code
- Use environment variables for secrets

### Abuse Prevention

- Rate limit votes per user
- Cooldown on card usage
- Detect and ignore spam patterns
- Optional moderator controls

---

## Accessibility

### Requirements

- **Screen reader support**: Semantic HTML, ARIA labels
- **Keyboard navigation**: All actions accessible via keyboard
- **Color blindness**: Don't rely solely on color (use icons, labels)
- **High contrast mode**: Support system preferences
- **Text scaling**: UI should work at 200% zoom
- **Captions**: Consider audio effects for accessibility

---

## Monitoring & Analytics

### Metrics to Track

- Questions answered (correct/incorrect)
- Cards used (frequency, effectiveness)
- Average completion time
- Win/loss rate
- Player engagement (chat participation)
- Technical: Error rates, crash logs, performance metrics

### Implementation

```typescript
interface Analytics {
  trackEvent(event: string, properties?: Record<string, any>): void
}

// Example usage
analytics.trackEvent('QUESTION_ANSWERED', {
  question_id: 'q001',
  correct: true,
  time_taken_seconds: 23,
  cards_active: ['yaffa_drone', 'palestine_action']
})

analytics.trackEvent('CARD_DEPLOYED', {
  card_id: 'operation_gates',
  votes_required: 3,
  time_to_consensus_seconds: 8
})
```

---

## Future Enhancements

### Phase 1+
- [ ] Sound effects and music
- [ ] Advanced animations
- [ ] Multiple language support
- [ ] Accessibility improvements

### Phase 2+
- [ ] Multiplayer competitive mode (streamer vs streamer)
- [ ] Leaderboards
- [ ] Achievement system
- [ ] Custom card creator

### Phase 3+
- [ ] Mobile app
- [ ] VR mode (?)
- [ ] AI opponent for solo play
- [ ] Community quiz marketplace

---

## Development Guidelines

### Framework Development Principles

**CRITICAL: Always ask "Is this framework code or content?"**

Before writing code, determine:
- ✅ **Framework code**: Generic, reusable across any quiz → Put in engine
- ❌ **Content/Game code**: Specific to Palestinian quiz → Put in data files

**Examples**:

```typescript
// ❌ BAD - Palestinian quiz logic in engine
function calculateScore(question: Question): number {
  if (question.topic === "1948_nakba") {
    return question.points * 1.5; // More points for Nakba questions
  }
  return question.points;
}

// ✅ GOOD - Generic scoring in engine, multipliers in data
function calculateScore(question: Question, config: GameConfig): number {
  const multiplier = config.topic_multipliers[question.topic] || 1.0;
  return question.points * multiplier;
}

// In content/palestinian-quiz/config.yml:
// topic_multipliers:
//   1948_nakba: 1.5
//   oslo_accords: 1.2
```

**Golden Rules**:
1. **No hardcoded quiz content** - questions, cards, topics must be external data
2. **No hardcoded game rules** - timer duration, passing grade, vote thresholds configurable
3. **No hardcoded card names/IDs** - engine interprets effects generically
4. **Build for multiple quizzes** - if it only works for Palestinian quiz, it's wrong

### Code Style

- **Functional over OOP** where appropriate
- **Pure functions**: Minimize side effects (easier to test, reuse)
- **Immutability**: Use const, avoid mutations
- **Small modules**: Max 500 lines per file (except inline tests)
- **Type safety**: Use TypeScript strict mode / Rust type system
- **No stubs**: Complete implementations only
- **Generic over specific**: Prefer `Card[]` over `PalestinianCard[]`
- **Configuration over code**: Prefer YAML/JSON config over constants

### Testing Framework Code

**Test with multiple content sets** to ensure genericity:

```typescript
describe('CardEffectResolver', () => {
  it('should work with any card definition', () => {
    // Test with Palestinian quiz cards
    const palestineCard = loadCard('content/palestinian-quiz/cards/yaffa_drone.yml');
    applyCard(palestineCard);
    expect(/* ... */);

    // Test with hypothetical science quiz cards
    const scienceCard = {
      id: 'periodic_table_helper',
      effects: [{ type: 'ELIMINATE_WRONG_ANSWER', count: 2 }]
    };
    applyCard(scienceCard);
    expect(/* ... same behavior */);
  });
});
```

### Documentation

**For Engine**:
- **README**: Setup, architecture, "How to create a quiz"
- **API docs**: All public interfaces
- **Effect Reference**: All supported card effect types
- **Schema docs**: Question/Card/Config schemas with examples
- **ADRs**: Why we chose certain designs

**For Reference Implementation**:
- **Content README**: Description of Palestinian quiz
- **Question sources**: Citations for historical facts
- **Card design notes**: Why certain cards exist, balance considerations

### Version Control

**Branch Naming**:
- **Engine features**: `feat/card-voting-system`, `feat/campaign-mode`
- **Content additions**: `content/palestine-part-4`, `content/science-quiz`
- **Bug fixes**: `fix/timer-drift`, `fix/vote-counting`

**Commit Messages**:
- `feat(engine): add configurable vote thresholds` ← Framework
- `content(palestine): add Lebanon War questions` ← Content
- `docs(api): document card effect schema`

**Separation**:
- Keep engine commits separate from content commits
- Engine changes should NEVER break existing content
- Content changes don't require engine releases

### Repository Structure

```
cgq/
├── engine/              # Framework code (core repo)
│   ├── src/
│   ├── tests/
│   └── docs/
├── content/             # Quiz content (can be separate repo)
│   ├── palestinian-quiz/
│   ├── science-quiz/
│   └── history-quiz/
├── tools/               # Build tools, validators
│   ├── cgq-cli/
│   └── quiz-editor/
└── examples/            # Example quizzes for tutorials
    └── minimal-quiz/
```
- **No uncommitted code**: Always run tests before committing

---

## Summary: Building a Framework, Not Just a Game

### What We're Building

**CGQ Engine** - A reusable framework for creating card-based quiz games
- Input: Quiz data (questions, cards, config) as JSON/YAML
- Output: Playable interactive quiz with Twitch integration
- Analogy: Unity for quiz games, RPG Maker for educational content

### What We're NOT Building

**A single Palestinian history quiz** - That's just our reference implementation
- The Palestinian quiz validates the framework
- It demonstrates what's possible
- It serves as example content for documentation

### Success Criteria

**We succeed when**:
1. A non-programmer can create a science quiz using our framework
2. The same engine runs both Palestinian and science quizzes without code changes
3. Content creators share quiz packs as data files, not forks of the engine
4. The framework is documented well enough that others can extend it

**We fail if**:
- Palestinian quiz logic is hardcoded in the engine
- Creating a new quiz requires modifying engine code
- The engine only works with one specific quiz format

### Key Architectural Principles

1. **Data-Driven**: Everything configurable via JSON/YAML
2. **Content-Agnostic**: Engine has no knowledge of quiz topics
3. **Composable**: Card effects combine like LEGO blocks
4. **Extensible**: Plugin system for custom effects
5. **Validatable**: Schemas ensure content correctness
6. **Distributable**: Content + Engine → Single binary

### The Vision

**Content creators should be able to**:
- Write quiz questions in a text editor
- Define custom cards with effects
- Configure game rules
- Run `cgq-cli build` and get a working game
- Share their quiz as a downloadable binary or data pack

**Without ever**:
- Writing TypeScript/Rust code
- Understanding the engine internals
- Forking the repository
- Waiting for us to add their content

This is a **platform**, not an application. Build it like one.

---

*Document created: 2026-01-15*
*Last updated: 2026-01-15*
*Type: Framework Technical Specification*
