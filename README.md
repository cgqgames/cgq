# CGQ Project Documentation

## Overview

This directory contains planning and technical documentation for the **CGQ (Card Game Quiz)** automation project.

CGQ is an educational quiz platform that combines multiple-choice questions with a card-based modifier system, designed for Twitch streaming with chat integration.

## Documents

### 📋 [planning.md](./planning.md)
**High-level project planning document**

Contains:
- Project overview and concept
- Game modes (Normal, Campaign, Aftermath)
- Core features and requirements
- Data schemas
- Development phases
- Success metrics
- Resource requirements
- Open questions

**Audience**: Project stakeholders, designers, new contributors

### 🔧 [technical-spec.md](./technical-spec.md)
**Detailed technical specification**

Contains:
- System architecture
- Component design
- Implementation details with code examples
- Twitch integration approach
- Testing strategy
- Deployment options
- Performance and security considerations

**Audience**: Developers, technical architects

## Quick Start for Developers

### Project Goals

**Current State**: Manual quiz operation requiring constant host intervention
- Timer management is manual
- Card effects applied manually
- Answer consensus determined manually
- High cognitive load on streamer

**Target State**: Fully automated quiz system
- Timer automatically pauses/resumes
- Cards deployed via chat voting
- Answers processed automatically
- Host can focus on commentary and engagement

### Key Automation Requirements

1. **Timer System**
   - Auto-pause when answer displayed
   - Auto-resume on next question
   - Modification via cards (+/- time)

2. **Card Voting**
   - Players type `use <card-name>` in chat
   - Visual vote counter (e.g., "2/3 votes")
   - Auto-deploy at threshold
   - Card effects apply immediately

3. **Answer Processing**
   - Players type A/B/C/D in chat
   - Two matching answers = consensus
   - Auto-lock and reveal answer
   - Score updated automatically

4. **Randomization**
   - Question order randomized each game
   - Answer positions (A/B/C/D) shuffled per question
   - Prevents memorization

5. **Campaign Mode**
   - Progressive difficulty
   - Store system for card purchases
   - Progression persistence
   - Visual level transitions

### Technology Recommendations

**Backend**: Rust or Go (prefer native-compiled single binary)
- Event-driven architecture
- Clean separation of concerns
- Strong type safety

**Frontend**: Web-based for OBS browser source
- React/Svelte/Vue
- WebSocket for real-time updates
- Responsive design

**Twitch**: IRC or official API
- Chat message parsing
- Command recognition
- Rate limiting

**Data**: JSON/YAML configuration files
- Quiz definitions
- Card definitions
- Campaign configs

### Development Phases

1. **MVP (Phase 1)**: Core quiz automation
   - Question/answer cycle
   - Basic timer system
   - Chat integration for answers
   - Simple UI

2. **Full Features (Phase 2)**: Complete card system
   - All card types
   - Voting mechanism
   - Visual effects
   - Audio feedback

3. **Campaign (Phase 3)**: Progression system
   - Campaign map
   - Store implementation
   - Save/load system
   - Multiple quiz packs

4. **Tools (Phase 4)**: Content creation
   - Quiz editor
   - Card creator
   - Campaign designer

5. **Polish (Phase 5)**: Production ready
   - Visual effects
   - Accessibility
   - Performance optimization
   - Documentation

## Project Structure (Proposed)

```
cgq/
├── backend/           # Game engine
│   ├── src/
│   │   ├── main.rs
│   │   ├── timer.rs
│   │   ├── cards.rs
│   │   ├── quiz.rs
│   │   ├── twitch.rs
│   │   └── state.rs
│   └── Cargo.toml
├── frontend/          # Web UI
│   ├── src/
│   │   ├── components/
│   │   ├── App.tsx
│   │   └── main.tsx
│   └── package.json
├── quizzes/           # Quiz data
│   ├── hundred_years_war_pt1.json
│   ├── hundred_years_war_pt2.json
│   └── ancient_history.json
├── cards/             # Card definitions
│   ├── resistance.json
│   ├── palestinian.json
│   ├── politics.json
│   └── negative.json
├── campaigns/         # Campaign configs
│   └── hundred_years_war.json
├── assets/            # Images, audio, etc.
│   ├── cards/
│   ├── maps/
│   └── sounds/
└── docs/              # Documentation
    ├── API.md
    └── CONTRIBUTING.md
```

## Current Status

**Phase**: Planning
**Team Size**: 3 developers (need more)
**Blockers**: None
**Next Steps**:
1. Finalize tech stack
2. Set up project repository
3. Create MVP proof of concept
4. Implement basic Twitch integration

## Contributing

See `technical-spec.md` for development guidelines including:
- Code style requirements
- Testing expectations
- Git workflow (feature branches, conventional commits)
- Documentation standards

## Key Insights from Transcript

### Pain Points (Manual Operation)

1. **Timer Management**: Streamer constantly forgets to pause/resume timer
   > "I forgot to set the timer... This is why I need this to be automated"

2. **Vote Tracking**: Difficult to track card votes mentally
   > "We need three people to agree to use this card"

3. **Cognitive Load**: Host tries to manage game, explain answers, and engage chat simultaneously
   > "This is the complicated thing about doing this quiz manually"

4. **Randomization**: Questions/answers in same order every time
   > "The order of questions could be randomized... so players don't just memorize which letter"

### Design Goals (From Streamer)

1. **Educational Focus**: Quiz based on books about Palestinian history
   - "The Hundred Years War on Palestine" by Rashid Khalidi
   - Multiple quiz packs planned for different topics

2. **Collaborative Gameplay**: Chat works together, not competitively
   - Consensus-based answering (2 matching votes)
   - Shared victory/defeat

3. **Strategic Depth**: Cards add decision-making layer
   - Eliminate wrong answers
   - Modify timer/points
   - Counter negative effects

4. **Progression**: Campaign mode provides long-term goals
   - Unlock cards gradually
   - Store upgrade system
   - Visual progression through map

5. **Visual Polish**: Time-of-day system, visual effects
   - Dawn → morning → noon → dusk → night
   - Gunfire effects for resistance cards
   - Animated vote indicators

6. **Accessibility**: Twitch integration is non-negotiable
   - Must work as OBS browser source
   - Simple chat commands
   - No complex setup for players

### Technical Insights

**Card System Complexity**:
- Permanent vs temporary cards
- Card slots (max 4)
- Countering/banning mechanics
- Conditional effects (e.g., "timer > 50%")
- Vote requirements can be modified by other cards
- Effects can stack or trigger additional cards

**Campaign Complexity**:
- 5 levels per campaign
- Store with 3 upgrade tiers
- Currency = surplus points (score above passing grade)
- Card purchases don't auto-deploy, just add to deck
- Store upgrades require depositing specific card types
- Visual storytelling through store keeper progression

**Timing Critical**:
- Questions have fixed total time (e.g., 16 minutes for 15 questions)
- Timer modifications significantly impact difficulty
- Balance is important (too easy/hard ruins experience)

## Questions for Team Discussion

1. **Platform**: Self-hosted on streamer's machine vs cloud-hosted multi-tenant?
2. **Language**: Rust for performance vs TypeScript for faster development?
3. **Persistence**: File-based saves vs database?
4. **Scope**: MVP only or plan for full feature set from start?
5. **Content Pipeline**: How to create new quizzes/cards efficiently?
6. **Testing**: Who will playtest for balance?
7. **Licensing**: Open source (what license?) or proprietary?

## Resources

**Source Material**:
- Transcript: `../cgq/share/transcripts/2026-01-15_NA_Kairos_Rev_ (live) 2026-01-15 10_06.txt`
- Stream: Kairos Rev (Twitch)
- Existing quiz content: Multiple books/topics prepared

**Team**:
- Streamer/Designer: Kairos Rev
- Programmers: 3 (seeking more)
- Artist: TBD
- Sound Designer: TBD (or use stock audio)

## Timeline Estimate

**Phase 1 (MVP)**: TBD
**Phase 2 (Full Features)**: TBD
**Phase 3 (Campaign)**: TBD
**Phase 4 (Tools)**: TBD
**Phase 5 (Polish)**: TBD

*Note: Following CLAUDE.md guidelines, we avoid specific time estimates. Focus is on what needs to be done, not when.*

## Contact

Project discussions happening on:
- Twitch stream: [Kairos Rev]
- Discord: TBD
- GitHub: TBD

---

*Documentation created: 2026-01-15*
*Based on live stream transcript analysis*
