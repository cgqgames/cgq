# CGQ (Card Game Quiz) - Framework Planning Document

## Project Overview

**CGQ is a general-purpose framework for creating interactive card-based quiz games**, not a single application. The goal is to build a flexible engine that content creators can use to build their own educational quiz experiences by providing rulesets, card definitions, and question content as data.

**The framework provides**:
- Core game engine with timer, scoring, and state management
- Card effect system with composable, data-driven mechanics
- Twitch chat integration for collaborative gameplay
- Rendering system for OBS/browser sources
- Campaign/progression system
- Content loading from configuration files

**The Palestinian history quiz described in the transcript is the first implementation** built using this framework, serving as both a proof-of-concept and a reference implementation.

### Vision: "The RPG Maker of Card Game Quizzes"

Just as RPG Maker allows creators to build role-playing games without programming, CGQ should enable educators and content creators to build interactive quiz experiences by:
1. Writing quiz questions in JSON/YAML
2. Defining custom cards with effects
3. Configuring game rules and mechanics
4. Compiling into a playable game

**Current State**: Manual operation requiring constant host intervention
**Target State**: Fully automated framework with content-driven game generation

---

## Core Concept

The quiz combines three elements:
1. **Multiple-choice questions** with varying point values
2. **Card system** that modifies gameplay (add/remove time, eliminate answers, change point values)
3. **Timer-based pressure** creating urgency and challenge

Players collaborate via Twitch chat to answer questions before time runs out. Success requires achieving a passing grade (typically 20 points) before the timer expires.

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

4. **Persistence** (Campaign mode)
   - Save player progression
   - Store state
   - Unlocked cards
   - Level completion

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

### Phase 3: Campaign Mode
**Goal**: Progression framework for multi-level games

**Engine Development**:
- [ ] Campaign data structure and loader
- [ ] Level progression engine
- [ ] Store system (generic purchasing/upgrading)
- [ ] Persistence/save system (game-agnostic)
- [ ] Campaign map renderer (configurable layouts)
- [ ] Difficulty scaling system
- [ ] Time-based visual effects engine

**Reference Implementation**:
- [ ] Palestinian quiz campaign (5 levels)
- [ ] Campaign map assets
- [ ] Store progression configuration

**Deliverable**:
- Campaign framework usable for any quiz topic
- Complete Palestinian quiz campaign
- Campaign creation guide

**Success Criteria**:
- ✅ Can create campaigns with different themes/maps
- ✅ Store system works with any card set

### Phase 4: Content Creation Tools
**Goal**: Tooling for non-programmers to create quizzes

**Engine Development**:
- [ ] Schema validation for quiz/card/config files
- [ ] Content compiler (validates and packages content)
- [ ] Asset management system
- [ ] Hot-reload for development

**Tools Development**:
- [ ] Quiz editor UI (web-based or desktop)
- [ ] Card creator tool with effect builder
- [ ] Campaign designer (visual map editor)
- [ ] Asset pipeline (image/audio import)
- [ ] Build tool (content → game binary)

**Documentation**:
- [ ] Comprehensive framework docs
- [ ] Tutorial: "Create Your First Quiz"
- [ ] Card effect reference
- [ ] Example quizzes (history, science, etc.)

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
