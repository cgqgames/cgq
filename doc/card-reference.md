# Card System Reference

## Overview

Cards modify gameplay by affecting timers, points, available answers, and other game mechanics. Cards are drawn randomly each question and can be deployed by player vote.

## Card Categories

### Permanence Types

**Permanent Cards**:
- Remain on table after deployment
- Occupy card slots (max 4 slots by default)
- Effects persist until used or game ends
- Examples: Palestine Action, Yaffa Drone Strike, Francesca Albanese

**Temporary Cards**:
- Take effect immediately when drawn
- Disappear on next question
- Don't occupy card slots
- Examples: 40 Beheaded Babies, Flour Massacre, Indiscriminate Bombing

### Faction Types

Cards belong to different factions that affect cost, availability, and interactions:

1. **Resistance Cards** (Positive)
   - Highest store cost
   - Powerful effects
   - Can counter IDF cards
   - Used for store upgrades in campaign

2. **Palestinian Cards** (Positive)
   - Medium store cost
   - Supportive effects
   - Provide utility

3. **Politics Cards** (Positive/Neutral)
   - Variable cost
   - Mixed effects (e.g., +time but -points)
   - Represent international political actions

4. **IDF Cards** (Negative)
   - Cannot be purchased
   - Drawn randomly
   - Harmful effects
   - Can be countered or banned

5. **Event Cards** (Negative)
   - Cannot be purchased
   - Represent historical events
   - Temporary negative effects

## Card Effects

### Timer Effects

**Add Time**
- Increases remaining time
- Example: Donald Trump (+1 minute)

**Subtract Time**
- Decreases remaining time
- Example: 40 Beheaded Babies (-1 minute)

### Point Effects

**Add Points**
- Increases current question value
- Example: Operation Gates of Hell (+2 points)

**Subtract Points**
- Decreases current question value
- Example: Donald Trump (-3 points)

**Set Points**
- Overrides current question value
- Example: Flour Massacre (sets to 1 point)

### Answer Effects

**Eliminate Wrong Answer**
- Removes incorrect options
- Makes question easier
- Example: Yaffa Drone Strike (1 answer), Francesca Albanese (1 answer)

### Meta Effects

**Counter Cards**
- Negates specific card types
- Example: Francesca Albanese (counters Hasbara cards)

**Ban Card Types**
- Prevents certain cards from appearing
- Example: Palestine Action (bans Reaper Drone, Gospel AI, Where's Daddy)

**Add Card Slot**
- Increases max permanent cards on table
- Example: Chinese Air Drop (+1 slot)

**Modify Vote Requirement**
- Changes votes needed to deploy cards
- Can reduce to 2, 1, or even 0
- Can go negative (enables deploying 2 cards in one question)

**Play Extra Card**
- Allows deploying another card this question
- Example: Dr. Rafat Al-Arir

## Card List (From Transcript)

### Resistance Cards (Positive, Permanent)

**Palestine Action**
- **Effect**: Bans specific IDF cards (Reaper Drone, Gospel AI, Where's Daddy)
- **Condition**: Timer must be >50%
- **Requires**: 3 votes
- **Store Cost**: 5 points (level 1)

**Yaffa Drone Strike**
- **Effect**: Eliminates 1 incorrect answer
- **Requires**: 3 votes
- **Store Cost**: 5 points (level 1)

**Operation Gates of Hell**
- **Effects**:
  - Eliminates 1 incorrect answer
  - +1 minute to timer
  - +2 points to current question
- **Counters**: Non-Reaper Drone IDF cards
- **Requires**: 3 votes
- **Store Cost**: 5 points (level 1)
- **Note**: Must be used BEFORE answering question

### Palestinian Cards (Positive, Permanent)

**Francesca Albanese**
- **Effects**:
  - Counters Hasbara cards
  - Eliminates 1 incorrect answer
- **Requires**: 3 votes
- **Store Cost**: 4 points (level 1)

**Dr. Rafat Al-Arir**
- **Effects**:
  - +2 points to current question
  - Play one more card this question (non-stackable)
- **Special**: Also activates if killed (unclear mechanic)
- **Requires**: 3 votes
- **Store Cost**: 4 points (level 1)

### Politics Cards (Positive/Mixed, Permanent)

**Donald Trump**
- **Effects**:
  - +1 minute to timer
  - -3 points to current question (!)
- **Cost to Use**: 2 points (must spend points to activate)
- **Requires**: 3 votes
- **Store Cost**: 3 points (level 1)
- **Note**: Mixed blessing - helps with time but reduces question value

**Chinese Air Drop**
- **Effect**: Adds 1 extra card slot to table
- **Requires**: 3 votes
- **Store Cost**: 3 points (level 1)
- **Note**: Increases max permanent cards from 4 to 5+

### IDF Cards (Negative, Temporary)

**Indiscriminate Bombing**
- **Effect**: Removes 1 Palestinian or Resistance card from table
- **Permanence**: Temporary (disappears next question)
- **Cannot be purchased**: Random draw only

**Reaper Drone**
- **Effect**: [Not specified in transcript]
- **Can be banned by**: Palestine Action
- **Permanence**: Likely temporary

**Gospel AI**
- **Effect**: [Not specified in transcript]
- **Can be banned by**: Palestine Action

**Where's Daddy**
- **Effect**: [Not specified in transcript]
- **Can be banned by**: Palestine Action

**Hasbara Card(s)**
- **Effect**: [Not specified in transcript]
- **Countered by**: Francesca Albanese

### Event Cards (Negative, Temporary)

**40 Beheaded Babies**
- **Effects**:
  - -1 point from current question (reduces to minimum of 1)
  - -1 minute from timer
- **Permanence**: Temporary (disappears next question)
- **Cannot be purchased**: Random draw only

**Flour Massacre**
- **Effect**: Sets current question value to 1 point
- **Permanence**: Temporary (disappears next question)
- **Cannot be purchased**: Random draw only

## Card Mechanics

### Drawing Cards

- One card drawn per question
- Random selection from deck
- Banned card types excluded from draw pool
- Campaign mode: only unlocked cards can be drawn

### Voting System

**Standard Voting**:
1. Player types `use <card-name>` in chat
2. System increments vote counter for that card
3. Visual indicator shows votes (e.g., "2/3")
4. At threshold (default 3), card auto-deploys
5. Votes reset each question

**Vote Requirements**:
- Default: 3 votes
- Can be modified by cards
- Can be reduced to 2, 1, or 0
- Negative requirements allow multiple cards per question

**Fuzzy Matching**:
- Partial names work (e.g., "yaffa" matches "Yaffa Drone Strike")
- Case-insensitive
- Ignore special characters

**Vote Rules**:
- One vote per user per card per question
- Duplicate votes from same user ignored
- Votes only valid during question phase (not after answer locked)
- Cannot use cards after answering question (would be cheating)

### Card Slots

**Slot System**:
- Default: 4 slots for permanent cards
- Cards occupy slots when deployed
- When all slots full, no more permanent cards can be deployed
- Temporary cards don't use slots
- Slots can be increased by cards (Chinese Air Drop)

**Slot States**:
- **Empty**: Available for new card
- **Occupied**: Has permanent card
- **Voted**: Card has votes but not deployed yet

### Card Interactions

**Countering**:
- Some cards negate others
- Example: Francesca Albanese counters Hasbara cards
- Countered card has no effect

**Banning**:
- Prevents card types from being drawn
- Example: Palestine Action bans Reaper Drone, Gospel AI, Where's Daddy
- Lasts for rest of game

**Removal**:
- Some negative cards remove positive cards from table
- Example: Indiscriminate Bombing removes Palestinian/Resistance card
- Frees up slot

**Stacking**:
- Multiple cards can affect same value
- Example: +2 points from one card, -3 from another = -1 net
- Effects apply in order drawn/deployed

## Campaign Mode Specifics

### Store Pricing

| Level | Slots | Resistance | Palestinian | Politics |
|-------|-------|------------|-------------|----------|
| 1     | 3     | 5          | 4           | 3        |
| 2     | 4     | 4          | 3           | 2        |
| 3     | 6     | 3          | 2           | 1        |

### Card Acquisition

**Purchase**:
- Spend surplus points (score above passing grade)
- Card added to deck (not auto-deployed)
- Becomes available in random draws

**Unlock vs Deploy**:
- Unlocked: Card can appear in random draws
- Deployed: Card is active on table
- Purchasing unlocks, doesn't auto-deploy

### Store Upgrades

**Requirements**:
- Deposit specific card types
- Example: 6 Resistance cards to upgrade level 1→2

**Benefits**:
- Lower prices
- More card slots in store offering
- Better selection

**Depositing Cards**:
- Must purchase card first
- Deposit removes from deck
- Cannot withdraw deposited cards
- Permanent upgrade investment

## Design Patterns

### Positive Cards

- Higher vote requirements (3 by default)
- Permanent (stay on table)
- Provide advantages
- Can be purchased in store
- Occupy card slots

### Negative Cards

- No vote requirement (auto-deploy)
- Temporary (disappear next question)
- Cannot be purchased
- Random draws only
- Don't occupy slots (usually)

### Balancing

**Time vs Points Trade-offs**:
- Donald Trump: +time but -points
- Creates strategic decision: need time or need points?

**Conditional Effects**:
- Palestine Action only works if timer >50%
- Encourages early deployment

**Scarcity**:
- Limited card slots (4 default)
- Must choose which cards to deploy
- Strategic resource management

**Cost to Activate**:
- Some cards cost points to use (Donald Trump: 2 points)
- Risk/reward decision

## Implementation Considerations

### Data Structure

```json
{
  "id": "yaffa_drone",
  "name": "Yaffa Drone Strike",
  "type": "resistance",
  "permanence": "permanent",
  "vote_requirement": 3,
  "cost": 5,
  "effects": [
    {
      "type": "eliminate_wrong_answer",
      "count": 1
    }
  ],
  "visual": {
    "image": "yaffa_drone.png",
    "sound": "drone_strike.mp3",
    "animation": "explosion"
  }
}
```

### Effect Processing Order

1. Card drawn (random)
2. If temporary, effects apply immediately
3. If permanent, card displayed for voting
4. Players vote via chat
5. At threshold, card deployed
6. Effects applied to game state
7. Visual/audio feedback
8. Card added to active cards (permanent) or removed (temporary)

### Special Cases

**Negative Vote Requirements**:
- If vote requirement becomes -1, can deploy 2 cards
- If -2, can deploy 3 cards
- Enables powerful combos

**Empty Slots**:
- If all 4 slots occupied, permanent cards cannot be deployed
- Players must use existing cards or wait for game to progress
- Encourages using cards rather than hoarding

**Effect Conflicts**:
- Multiple cards set question to different point values
- Resolution: Last deployed wins? Sum effects? Average?
- **Need clarification from designer**

## Open Questions

1. **Card removal**: When Indiscriminate Bombing removes a card, which one? Random? Player choice?
2. **Effect priority**: If multiple cards modify same value, what order?
3. **Dr. Rafat Al-Arir "activates if killed"**: What does this mean? Respawn? Auto-deploy?
4. **Hasbara cards**: What are their effects?
5. **Other IDF cards**: Effects not described in transcript
6. **Vote requirement minimum**: Can it go below 0? What's the limit?
7. **Card slot maximum**: Can it exceed some cap? (e.g., 10 slots)
8. **Store randomization**: How are store offerings selected? Pure random? Weighted?

## Visual Design Notes

**Card UI Elements**:
- Card image/illustration
- Card name
- Card type (color-coded?)
- Effect description
- Vote counter (pie chart or "2/3")
- Slot indicator (which slot it occupies)

**Animations**:
- Card draw: Slide in from deck
- Card deploy: Flash/glow effect
- Card removal: Fade out
- Resistance cards: Gunfire, muzzle flash
- Vote cast: Pie chart fills

**Color Scheme**:
- Resistance: Green?
- Palestinian: Red/white/black (flag colors)?
- Politics: Blue?
- IDF: Gray/military colors?
- Event: Yellow/warning colors?

---

*Last updated: 2026-01-15*
*Source: Kairos Rev stream transcript*
