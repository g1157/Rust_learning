# Story 002: Mode Selection Page Categorization - Brownfield Enhancement

## User Story

As a **player**,
I want **the mode selection menu to be organized into clear categories (Game Modes, Achievements, Settings)**,
So that **I can quickly find what I'm looking for and understand the menu structure**.

## Story Context

**Existing System Integration:**

- Integrates with: `draw_mode_selection()` in `src/ui.rs:598-938`
- Technology: Macroquad rendering, custom `ModeCardParams` system
- Follows pattern: Existing card rendering with `draw_mode_card()`
- Touch points: UI layout calculations, card positioning logic

**Current State:**
All 7 cards (Survival, Duel, TimeAttack, Roguelike, Online, Achievements, Settings) are rendered as a single vertical list with identical styling, making it difficult to distinguish functional categories.

## Proposed Categorization

| Category | Items | Visual Treatment |
|----------|-------|------------------|
| **Game Modes** | Survival, Duel, TimeAttack, Roguelike, Online | Main section, game-themed colors |
| **Progress** | Achievements | Separate section, gold accent |
| **System** | Settings | Footer section, subtle green accent |

## Acceptance Criteria

**Functional Requirements:**

1. Game mode cards grouped together with section header "Game Modes"
2. Achievements card in separate "Progress" section with visual divider
3. Settings card in "System" section at bottom with visual divider
4. Section headers clearly visible with appropriate styling

**Integration Requirements:**

5. Existing card selection/navigation logic unchanged (Up/Down/W/S)
6. Card rendering uses existing `draw_mode_card()` function
7. Responsive layout calculations preserved
8. All existing functionality (Enter to select, indicators) maintained

**Quality Requirements:**

9. Visual hierarchy clearly distinguishable
10. Text remains readable on all screen sizes
11. No performance regression from additional draw calls

## Technical Notes

- **Integration Approach:** Add section headers between card groups, adjust Y positioning
- **Existing Pattern Reference:** Title/subtitle rendering at lines 659-752
- **Key Constraints:** Must maintain responsive scaling (lines 611-642)

**Proposed Layout Structure:**
```
┌─────────────────────────────────────┐
│      "Choose Your Adventure"         │
│      (subtitle)                       │
├─────────────────────────────────────┤
│  ══ Game Modes ══                    │
│  [Survival Card]                     │
│  [Duel Card]                         │
│  [TimeAttack Card]                   │
│  [Roguelike Card]                    │
│  [Online Card]                       │
├─────────────────────────────────────┤
│  ══ Progress ══                      │
│  [Achievements Card]                 │
├─────────────────────────────────────┤
│  ══ System ══                        │
│  [Settings Card]                     │
└─────────────────────────────────────┘
```

**Code Change Location:**
```
src/ui.rs:598-938 (draw_mode_selection function)
```

**Implementation Steps:**

1. Create `draw_section_header()` helper function
2. Calculate section positions based on card counts
3. Insert section headers at appropriate Y positions
4. Adjust card Y offsets to account for headers
5. Add subtle divider lines between sections

## Risk and Compatibility Check

**Minimal Risk Assessment:**

- **Primary Risk:** Layout breaking on edge-case screen sizes
- **Mitigation:** Test on multiple resolutions (800x600 to 1920x1080)
- **Rollback:** Revert to single-list layout

**Compatibility Verification:**

- [x] No breaking changes to existing APIs
- [x] No database changes
- [x] UI changes follow existing design patterns (card system)
- [x] Performance impact is negligible (few extra text draws)

## Definition of Done

- [ ] Section headers rendered for each category
- [ ] Visual dividers between sections
- [ ] Card navigation still works correctly (7 items, looping)
- [ ] Responsive scaling maintained
- [ ] Manual test on 800x600, 1024x768, 1920x1080
- [ ] No regression in card selection highlighting

## Effort Estimate

**Complexity:** Medium (layout restructuring)
**Estimated Time:** 1-2 hours
