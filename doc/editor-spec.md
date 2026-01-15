# CGQ Builder - Visual Editor Specification

## Overview

**CGQ Builder** is a visual content creation tool that allows non-programmers to create card-based interactive games without writing code. It provides specialized editors for different game types (Quiz, Grid, Deck-builder) and a universal card editor that works across all game types.

**Key Principle**: If you can use PowerPoint or Google Forms, you should be able to use CGQ Builder.

---

## Architecture

### Application Structure

```
┌─────────────────────────────────────────────────────┐
│              CGQ Builder (Electron App)             │
├─────────────────────────────────────────────────────┤
│                                                     │
│  ┌──────────────┐  ┌───────────────────────────┐  │
│  │   Project    │  │     Content Editors       │  │
│  │   Manager    │  │                           │  │
│  │              │  │  - Card Editor            │  │
│  │  - New       │  │  - Quiz Editor            │  │
│  │  - Open      │  │  - Grid Editor            │  │
│  │  - Save      │  │  - Campaign Designer      │  │
│  │  - Export    │  │  - Config Editor          │  │
│  └──────┬───────┘  └──────────┬────────────────┘  │
│         │                     │                    │
│         └─────────┬───────────┘                    │
│                   │                                │
│         ┌─────────▼──────────┐                     │
│         │   Asset Manager    │                     │
│         │  - Images          │                     │
│         │  - Audio           │                     │
│         │  - Data files      │                     │
│         └─────────┬──────────┘                     │
│                   │                                │
│         ┌─────────▼──────────┐                     │
│         │  Preview Engine    │                     │
│         │  (CGQ Engine API)  │                     │
│         └────────────────────┘                     │
└─────────────────────────────────────────────────────┘
```

### Technology Stack

**Frontend**:
- **Framework**: React 18+ with TypeScript
- **UI Library**: Material-UI or Ant Design
- **Visual Editors**:
  - **Node-based**: React Flow (for card effect chains)
  - **Grid Editor**: Konva.js or Fabric.js (canvas)
  - **Code Editor**: Monaco Editor (VS Code editor component)
  - **Form Builder**: React Hook Form
  - **Rich Text**: Lexical or TipTap
- **State Management**: Zustand or Redux Toolkit
- **File System**: Electron IPC + Node.js fs

**Backend/Engine Integration**:
- **Preview Mode**: CGQ Engine running in background process
- **Validation**: Zod or AJV for schema validation
- **Build Process**: Custom YAML/JSON compiler

**Desktop Wrapper**:
- **Electron**: For cross-platform desktop app
- **Auto-Update**: electron-updater

---

## Core Features

### 1. Project Management

#### Project Structure

```
my-quiz-project/
├── project.cgq.json        # Project metadata
├── cards/
│   ├── yaffa_drone.yml
│   ├── time_warp.yml
│   └── radar_sweep.yml
├── content/
│   ├── quiz/
│   │   └── questions.yml
│   ├── grid/
│   │   └── layout.yml
│   └── config.yml
├── campaign/
│   └── levels.yml
├── assets/
│   ├── images/
│   ├── audio/
│   └── maps/
└── build/
    └── game.cgq.bundle
```

#### Project Metadata

```json
{
  "name": "Palestinian History Quiz",
  "version": "1.0.0",
  "game_type": "quiz",
  "author": "Kairos Rev",
  "description": "Educational quiz on Palestinian history",
  "created": "2026-01-15T00:00:00Z",
  "modified": "2026-01-15T12:00:00Z",
  "cgq_version": "0.1.0",
  "settings": {
    "default_language": "en",
    "enable_campaign": true,
    "enable_store": true
  }
}
```

#### Project Operations

- **New Project**: Wizard with templates (Blank, Quiz, Grid, Example)
- **Open Project**: File browser, recent projects list
- **Save**: Auto-save every 30s, manual save (Ctrl+S)
- **Export**: Bundle project → .cgq.bundle file
- **Import**: Load .cgq.bundle or YAML files

---

### 2. Universal Card Editor

The card editor is **game-type-agnostic** - same UI works for Quiz, Grid, Deck-builder cards.

#### UI Layout

```
┌──────────────────────────────────────────────────────────┐
│  Card Editor: Yaffa Drone Strike           [Save] [Test] │
├────────────┬─────────────────────────────────────────────┤
│            │                                             │
│  Card List │  ┌─ Basic Info ──────────────────────────┐  │
│            │  │                                        │  │
│  [Search]  │  │  Name: [Yaffa Drone Strike        ]   │  │
│            │  │  ID:   [yaffa_drone_strike        ]   │  │
│  ✓ Yaffa   │  │  Type: [Resistance ▼]                 │  │
│    Drone   │  │  Cost: [3] points                     │  │
│  □ Time    │  │  Votes Required: [3]──────             │  │
│    Warp    │  │                                        │  │
│  □ Radar   │  └────────────────────────────────────────┘  │
│    Sweep   │                                             │
│            │  ┌─ Effects ────────────────────────────┐  │
│ [+ New     │  │                                      │  │
│   Card]    │  │  Effect #1: Eliminate Wrong Answer   │  │
│            │  │  ┌────────────────────────────────┐  │  │
│            │  │  │ [Interceptor Builder]          │  │  │
│            │  │  │                                │  │  │
│            │  │  │ What to intercept:             │  │  │
│            │  │  │  Component: [Question      ▼]  │  │  │
│            │  │  │  Operation: [get_options   ▼]  │  │  │
│            │  │  │  When:      [Always        ▼]  │  │  │
│            │  │  │                                │  │  │
│            │  │  │ What to do:                    │  │  │
│            │  │  │  [Visual Builder] [Code View]  │  │  │
│            │  │  │                                │  │  │
│            │  │  │  ┌──────────────────────────┐ │  │  │
│            │  │  │  │ 1. Filter options        │ │  │  │
│            │  │  │  │    Keep: incorrect only  │ │  │  │
│            │  │  │  │                          │ │  │  │
│            │  │  │  │ 2. Select one            │ │  │  │
│            │  │  │  │    Method: random        │ │  │  │
│            │  │  │  │                          │ │  │  │
│            │  │  │  │ 3. Remove from list      │ │  │  │
│            │  │  │  └──────────────────────────┘ │  │  │
│            │  │  │                                │  │  │
│            │  │  │ Timing: ○ Before ● After      │  │  │
│            │  │  │ Priority: [100]──────          │  │  │
│            │  │  │ Duration: [One-time ▼]        │  │  │
│            │  │  └────────────────────────────────┘  │  │
│            │  │                                      │  │
│            │  │  [+ Add Effect]                      │  │
│            │  └──────────────────────────────────────┘  │
│            │                                             │
│            │  ┌─ Visual Assets ──────────────────────┐  │
│            │  │  Card Art: [📁 Upload] [🖼 Preview]   │  │
│            │  │  Sound:    [🔊 Select] [▶ Play]       │  │
│            │  └──────────────────────────────────────┘  │
│            │                                             │
│            │  ┌─ Preview ────────────────────────────┐  │
│            │  │  [▶ Test This Card]                   │  │
│            │  │  Shows: Question → Card activates     │  │
│            │  └──────────────────────────────────────┘  │
└────────────┴─────────────────────────────────────────────┘
```

#### Interceptor Builder - Visual Mode

**Step 1: Choose what to intercept**

```
Component: [Dropdown list of all components]
  - Question
  - Score
  - Timer
  - Grid
  - Position
  - Resources
  - ... (auto-populated from game type)

Operation: [Dropdown filtered by component]
  If Component = Question:
    - get_current
    - get_options
    - get_points
  If Component = Score:
    - get
    - add
    - multiply
  If Component = Grid:
    - get_cell
    - get_cells
    - reveal_cell
  ... etc.

When: [Condition builder]
  - Always
  - When timer > [50]%
  - When score < [10] points
  - Custom: [Expression builder]
```

**Step 2: Choose what to do (Transform)**

```
Transform Type: [Dropdown]
  - Modify Value (for numbers)
  - Modify Collection (for arrays)
  - Modify Object (for objects)
  - Custom Code

[Based on selection, show template-based builder]
```

**Template: Modify Collection**

```
┌────────────────────────────────────┐
│ Input: result (array)              │
│                                    │
│ Steps:                             │
│  1. [Filter        ▼]              │
│     Keep: [incorrect options ▼]    │
│     ┌─ Conditions ────────────┐    │
│     │ field.correct == false  │    │
│     └─────────────────────────┘    │
│                                    │
│  2. [Select        ▼]              │
│     Method: [random ▼]             │
│     Count:  [1]                    │
│                                    │
│  3. [Remove        ▼]              │
│     From: original list            │
│                                    │
│  [+ Add Step]                      │
│                                    │
│ Output: modified array             │
└────────────────────────────────────┘
```

**Template: Modify Value**

```
┌────────────────────────────────────┐
│ Input: result (number)             │
│                                    │
│ Operation: [Multiply ▼]            │
│   - Add                            │
│   - Subtract                       │
│   - Multiply                       │
│   - Divide                         │
│   - Set to                         │
│                                    │
│ Value: [2]                         │
│                                    │
│ Result: result * 2                 │
└────────────────────────────────────┘
```

**Code View** (for advanced users):

```javascript
// Toggle between Visual and Code mode
function transform(query, result) {
  const incorrect = result.filter(opt => !opt.correct);
  if (incorrect.length === 0) return result;
  const idx = Math.floor(Math.random() * incorrect.length);
  return result.filter((_, i) => result.indexOf(incorrect[idx]) !== i);
}
```

#### Effect Template Library

Pre-built templates users can insert:

**Common Card Effects**:
- **Double Points**: Multiply score additions by 2
- **Add Time**: Add X seconds to timer
- **Eliminate Answer**: Remove random wrong option
- **Reveal Cells**: Show grid cells within radius
- **Skip Question**: Move to next question
- **Extra Vote**: Reduce vote requirement by 1
- **Shield**: Block next negative card

**Template Selector**:

```
┌─────────────────────────────────────────┐
│  Effect Templates                       │
├─────────────────────────────────────────┤
│  Search: [_______________]          🔍  │
│                                         │
│  📂 Score Modifiers                     │
│    • Double Points                      │
│    • Add Bonus Points                   │
│    • Point Multiplier                   │
│                                         │
│  📂 Timer Modifiers                     │
│    • Add Time                           │
│    • Pause Timer                        │
│    • Speed Up/Slow Down                 │
│                                         │
│  📂 Question Helpers                    │
│    • Eliminate Wrong Answer             │
│    • Highlight Correct Area             │
│    • Show Hint                          │
│                                         │
│  📂 Grid Modifiers (for Grid games)     │
│    • Reveal Adjacent Cells              │
│    • Sonar Ping                         │
│    • Shield Cell                        │
│                                         │
│  [Use Template]  [Customize]            │
└─────────────────────────────────────────┘
```

When user selects template, it auto-fills the interceptor builder with appropriate values.

#### Card Validation

Real-time validation as user builds card:

```
✓ Card name is unique
✓ At least one effect defined
⚠ No artwork uploaded (optional but recommended)
✗ Transform function has syntax error on line 3
✓ Effect intercepts valid component
```

#### Card Preview/Testing

```
┌─────────────────────────────────────────┐
│  Test Card: Yaffa Drone Strike          │
├─────────────────────────────────────────┤
│  Scenario: Quiz Game                    │
│                                         │
│  Before card activation:                │
│  Question: What year...?                │
│  Options:                               │
│    A) 1948 ✓                            │
│    B) 1967                              │
│    C) 1973                              │
│    D) 1982                              │
│                                         │
│  [▶ Activate Card]                      │
│                                         │
│  After card activation:                 │
│  Question: What year...?                │
│  Options:                               │
│    A) 1948 ✓                            │
│    B) 1967                              │
│    C) 1973                              │
│  (Option D removed)                     │
│                                         │
│  ✓ Effect worked as expected            │
└─────────────────────────────────────────┘
```

---

### 3. Quiz Editor

Specialized editor for creating quiz questions.

#### UI Layout

```
┌──────────────────────────────────────────────────────────┐
│  Quiz Editor: Palestinian History Part 1      [+ Import] │
├────────────┬─────────────────────────────────────────────┤
│            │  Question 3 of 15                  [< >]    │
│ Questions  │                                             │
│            │  ┌─ Question Text ─────────────────────┐    │
│  1. ✓      │  │                                     │    │
│  2. ✓      │  │ How is the Dahiya doctrine a       │    │
│  3. ←      │  │ violation of the 1899 annex of the │    │
│  4.        │  │ Hague Convention?                  │    │
│  5.        │  │                                     │    │
│  ...       │  │ [B] Bold  [I] Italic  [Link]  [Img] │    │
│  15.       │  └─────────────────────────────────────┘    │
│            │                                             │
│ [+ Add     │  ┌─ Answer Options ───────────────────┐    │
│  Question] │  │                                     │    │
│            │  │  ● A: [It targets civilians    ] ✓  │    │
│            │  │  ○ B: [Violates sovereignty   ] ✗  │    │
│ [Import    │  │  ○ C: [Uses banned weapons    ] ✗  │    │
│  CSV]      │  │  ○ D: [Ignores diplomacy      ] ✗  │    │
│            │  │                                     │    │
│            │  │  [Randomize Options]                │    │
│            │  └─────────────────────────────────────┘    │
│            │                                             │
│            │  Points: [2]──────  Difficulty: [Medium ▼]  │
│            │                                             │
│            │  ┌─ Explanation ────────────────────────┐   │
│            │  │ According to Article 25 of the Hague │   │
│            │  │ Convention, the attack and bombardme │   │
│            │  │ ...                                  │   │
│            │  └──────────────────────────────────────┘   │
│            │                                             │
│            │  Source: [The Hague Convention of 1899]     │
│            │  Tags: [treaty] [1899] [war_crimes]         │
│            │                                             │
│            │  [Delete]  [Duplicate]  [Save]              │
└────────────┴─────────────────────────────────────────────┘
```

#### Bulk Import from CSV

Users can prepare questions in spreadsheet and import:

**CSV Format**:
```csv
Question,Option A,Option B,Option C,Option D,Correct,Points,Explanation,Source
"What year was X?","1948","1967","1973","1982","A",2,"Because...","Book Title"
```

**Import Dialog**:
```
┌─────────────────────────────────────────┐
│  Import Questions from CSV              │
├─────────────────────────────────────────┤
│  File: [Browse...] questions.csv        │
│                                         │
│  Preview:                               │
│  ┌─────────────────────────────────┐   │
│  │ 15 questions found              │   │
│  │ ✓ All required columns present  │   │
│  │ ✓ All rows valid                │   │
│  │ ⚠ 3 rows missing explanations   │   │
│  └─────────────────────────────────┘   │
│                                         │
│  Options:                               │
│  ☑ Append to existing questions         │
│  ☐ Replace all questions                │
│  ☑ Randomize option order               │
│                                         │
│  [Cancel]  [Import 15 Questions]        │
└─────────────────────────────────────────┘
```

---

### 4. Grid Editor

For creating Battleship-style grid-based games.

#### UI Layout

```
┌──────────────────────────────────────────────────────────┐
│  Grid Editor: Naval Battle                     [Export]  │
├────────────┬─────────────────────────────────────────────┤
│            │  Grid Configuration                         │
│ Tools      │  Size: [10] × [10]                          │
│            │                                             │
│ □ Select   │  ┌─────────────────────────────────┐       │
│ ✓ Place    │  │ 0 1 2 3 4 5 6 7 8 9             │       │
│   Ship     │  │ ┌─┬─┬─┬─┬─┬─┬─┬─┬─┬─┐           │       │
│ □ Obstacle │  │0│ │ │█│█│█│█│█│ │ │ │           │       │
│ □ Special  │  │ ├─┼─┼─┼─┼─┼─┼─┼─┼─┼─┤           │       │
│   Cell     │  │1│ │ │ │ │ │ │ │ │ │ │           │       │
│ □ Erase    │  │ ├─┼─┼─┼─┼─┼─┼─┼─┼─┼─┤           │       │
│            │  │2│█│ │ │ │ │ │ │ │ │ │           │       │
│ Layers     │  │ ├─┼─┼─┼─┼─┼─┼─┼─┼─┼─┤           │       │
│ ☑ Ships    │  │3│█│ │ │ │ │ │ │█│ │ │           │       │
│ ☑ Obstacles│  │ ├─┼─┼─┼─┼─┼─┼─┼─┼─┼─┤           │       │
│ ☐ Fog      │  │4│█│ │ │ │ │ │ │█│ │ │           │       │
│            │  │ ├─┼─┼─┼─┼─┼─┼─┼─┼─┼─┤           │       │
│ Ships      │  │5│ │ │ │ │ │ │ │█│ │ │           │       │
│ Carrier(5) │  │ ├─┼─┼─┼─┼─┼─┼─┼─┼─┼─┤           │       │
│   [1/1]    │  │6│ │ │ │ │ │ │ │ │ │ │           │       │
│ Battleship │  │ ├─┼─┼─┼─┼─┼─┼─┼─┼─┼─┤           │       │
│  (4) [2/2] │  │7│ │ │ │ │ │ │ │ │ │ │           │       │
│ Cruiser(3) │  │ ├─┼─┼─┼─┼─┼─┼─┼─┼─┼─┤           │       │
│   [0/3]    │  │8│ │ │ │ │ │ │ │ │ │ │           │       │
│            │  │ ├─┼─┼─┼─┼─┼─┼─┼─┼─┼─┤           │       │
│ [+ Add     │  │9│ │ │ │ │ │ │ │ │ │ │           │       │
│  Ship Type]│  │ └─┴─┴─┴─┴─┴─┴─┴─┴─┴─┘           │       │
│            │  └─────────────────────────────────┘       │
│            │                                             │
│            │  Cell Properties (0, 2):                    │
│            │  Type: [Ship ▼]                             │
│            │  Revealed: ☐  Occupied: ☑                   │
│            │  Special: ☐  [Treasure, Bonus Point, etc.]  │
│            │                                             │
│            │  [Validate Grid]  [Test Play]               │
└────────────┴─────────────────────────────────────────────┘
```

#### Grid Validation

```
✓ All ships placed
✓ No overlapping ships
✓ Grid is playable
⚠ No special cells defined (optional)
✓ Ready to export
```

---

### 5. Campaign Designer

Visual tool for creating multi-level campaigns.

#### UI Layout

```
┌──────────────────────────────────────────────────────────┐
│  Campaign Designer: Hundred Years War                    │
├────────────┬─────────────────────────────────────────────┤
│            │  Campaign Map                               │
│ Levels     │  ┌─────────────────────────────────────┐   │
│            │  │                                     │   │
│ 1. Dawn ✓  │  │     [Background: Gaza Map]          │   │
│ 2. Morning │  │                                     │   │
│ 3. Noon    │  │  🏁──●──────●──────●──────●──────●  │   │
│ 4. Dusk    │  │     Lvl1   Lvl2   Lvl3   Lvl4  Lvl5│   │
│ 5. Night   │  │   (Dawn)(Morning)(Noon)(Dusk)(Night)│   │
│            │  │                                     │   │
│ [+ Add     │  │  [Click level to edit]              │   │
│  Level]    │  └─────────────────────────────────────┘   │
│            │                                             │
│ Store      │  ┌─ Level 1 Configuration ─────────────┐   │
│ Level 1    │  │  Name: Dawn                         │   │
│ Level 2    │  │  Quiz: [Select Quiz ▼]              │   │
│ Level 3    │  │  Difficulty: [1]──────              │   │
│            │  │  Time of Day: [Dawn ▼]              │   │
│            │  │  Map Position: X:[100] Y:[50]       │   │
│            │  │  Negative Card Multiplier: [0.5]──  │   │
│            │  └─────────────────────────────────────┘   │
│            │                                             │
│            │  ┌─ Store Progression ─────────────────┐   │
│            │  │  Level 1: 3 slots, prices: 5/4/3   │   │
│            │  │  Level 2: 4 slots, prices: 4/3/2   │   │
│            │  │  Level 3: 6 slots, prices: 3/2/1   │   │
│            │  │                                     │   │
│            │  │  [Edit Store Levels]                │   │
│            │  └─────────────────────────────────────┘   │
└────────────┴─────────────────────────────────────────────┘
```

---

### 6. Asset Manager

Centralized asset management for all project resources.

```
┌──────────────────────────────────────────────────────────┐
│  Asset Manager                          [Upload] [Folder]│
├────────────┬─────────────────────────────────────────────┤
│            │  📁 Images                                  │
│ Filter     │  ┌─────────────────────────────────────┐   │
│ [All ▼]    │  │ ┌───────┐ ┌───────┐ ┌───────┐     │   │
│            │  │ │ card1 │ │ card2 │ │ map1  │     │   │
│ Search:    │  │ │🖼     │ │🖼     │ │🖼     │     │   │
│ [____]  🔍 │  │ │512x512│ │512x512│ │1920x  │     │   │
│            │  │ │PNG    │ │PNG    │ │PNG    │     │   │
│ Types      │  │ └───────┘ └───────┘ └───────┘     │   │
│ ☑ Images   │  └─────────────────────────────────────┘   │
│ ☑ Audio    │                                             │
│ ☑ Data     │  📁 Audio                                   │
│            │  ┌─────────────────────────────────────┐   │
│ Sort       │  │ 🔊 gunfire.mp3      [▶ Play] 2.3s  │   │
│ [Name ▼]   │  │ 🔊 victory.mp3      [▶ Play] 5.1s  │   │
│            │  │ 🔊 drone_strike.mp3 [▶ Play] 1.8s  │   │
│            │  └─────────────────────────────────────┘   │
│            │                                             │
│            │  Selected: card1.png                        │
│            │  ┌─ Properties ────────────────────────┐   │
│            │  │ Size: 512x512                       │   │
│            │  │ File Size: 125 KB                   │   │
│            │  │ Format: PNG                         │   │
│            │  │ Used in: 2 cards                    │   │
│            │  │  - Yaffa Drone Strike               │   │
│            │  │  - Operation Gates                  │   │
│            │  │                                     │   │
│            │  │ [Rename] [Delete] [Export]          │   │
│            │  └─────────────────────────────────────┘   │
└────────────┴─────────────────────────────────────────────┘
```

---

### 7. Preview/Playtest Mode

Test game without exporting.

```
┌──────────────────────────────────────────────────────────┐
│  Preview Mode                              [Stop] [Reset]│
├──────────────────────────────────────────────────────────┤
│                                                          │
│  [Actual game rendering here - same as player sees]     │
│                                                          │
│  Question 3/15                          Timer: 14:23     │
│  Score: 8 / 20                                           │
│                                                          │
│  How is the Dahiya doctrine...                           │
│                                                          │
│  A) It targets civilians                                 │
│  B) It violates territorial sovereignty                  │
│  C) It uses banned weapons                               │
│                                                          │
│  Active Cards: [Yaffa Drone] [Time Warp]                 │
│                                                          │
├──────────────────────────────────────────────────────────┤
│  Debug Panel:                                            │
│  Current State: WAITING_FOR_ANSWER                       │
│  Active Effects: 2                                       │
│  Last Event: CARD_DEPLOYED (yaffa_drone)                 │
│  [View State JSON]                                       │
└──────────────────────────────────────────────────────────┘
```

---

### 8. Export/Build System

Generate production-ready game bundles.

```
┌─────────────────────────────────────────┐
│  Export Project                         │
├─────────────────────────────────────────┤
│  Export Format:                         │
│  ● Standalone Binary (.exe/.app/.bin)  │
│  ○ Web Bundle (HTML/JS)                 │
│  ○ Content Pack (.cgq.bundle)           │
│                                         │
│  Target Platform:                       │
│  ☑ Windows                              │
│  ☑ macOS                                │
│  ☑ Linux                                │
│                                         │
│  Options:                               │
│  ☑ Include all assets                   │
│  ☑ Minify/compress                      │
│  ☐ Development mode (debug logs)        │
│                                         │
│  Output Directory:                      │
│  [Browse...] /path/to/output            │
│                                         │
│  [Cancel]  [Export]                     │
└─────────────────────────────────────────┘

Building...
[████████████████████░░░░] 80%
Compiling content files...
Bundling assets...
```

---

## Workflow Examples

### Example 1: Create a New Card in 2 Minutes

1. Click **"+ New Card"**
2. Name it **"Double Points"**
3. Select type **"Politics"**
4. Click **"Add Effect" → "From Template"**
5. Choose **"Score Modifiers → Double Points"**
6. Template auto-fills:
   - Component: Score
   - Operation: add
   - Transform: Multiply by 2
7. Upload card artwork (optional)
8. Click **"Save"**
9. Done!

### Example 2: Create a Quiz in 10 Minutes

1. **New Project** → Choose "Quiz Template"
2. In **Quiz Editor**, click **"Import CSV"**
3. Select `questions.csv` with 15 questions
4. Preview shows all questions imported correctly
5. Click **"Import"**
6. Tweak 2-3 questions manually (fix typos, add explanations)
7. In **Config Editor**, set passing grade to 20, timer to 16 minutes
8. Click **"Preview"** to test
9. Play through a few questions to verify
10. Click **"Export" → "Standalone Binary"**
11. Done - share the .exe with friends!

### Example 3: Create a Grid Game Card

1. In **Card Editor**, create new card **"Radar Sweep"**
2. Add Effect → **"From Template"** → **"Grid Modifiers → Reveal Adjacent"**
3. Template asks: **"Reveal radius?"** → Enter `1` (1 cell in each direction)
4. Template auto-generates:
   ```
   Component: Grid
   Operation: get_cells
   Transform: Reveal cells within distance 1
   ```
5. Upload radar icon as card art
6. **Test in Preview**: Select grid cell → Card reveals surrounding cells
7. Works! Click **"Save"**

---

## Success Metrics

**CGQ Builder succeeds when**:
- ✅ Non-programmers can create a playable quiz in <10 minutes
- ✅ Users can create complex card effects without writing code
- ✅ 90% of cards can be created using templates (no custom code needed)
- ✅ Users can preview/test without leaving the editor
- ✅ Exported games run identically to preview
- ✅ Editor is intuitive enough that users don't need to watch tutorials

**CGQ Builder fails if**:
- ❌ Users need to write code for basic cards
- ❌ No way to test cards without exporting
- ❌ Editor crashes or loses work
- ❌ Users can't figure out how to create their first quiz

---

## Implementation Priority

**Phase 1 - MVP** (Essential for first release):
- [x] Project management (new, open, save)
- [x] Card editor with template library
- [x] Quiz editor (manual entry)
- [x] Asset manager (basic upload/organize)
- [x] Preview mode
- [x] Export to YAML/JSON

**Phase 2** (Improve usability):
- [ ] CSV import for quiz questions
- [ ] Visual transform builder (no-code)
- [ ] Grid editor
- [ ] Campaign designer
- [ ] Validation and error checking
- [ ] Auto-save

**Phase 3** (Polish):
- [ ] Real-time preview updates
- [ ] Undo/redo
- [ ] Template gallery expansion
- [ ] Collaboration features
- [ ] Cloud save/sync
- [ ] Standalone binary export (Electron packaging)

---

*Last updated: 2026-01-15*
*Type: Visual Editor Specification*
