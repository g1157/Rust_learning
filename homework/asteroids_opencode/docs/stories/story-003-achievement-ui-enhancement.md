# Story 003: Achievement System UI Enhancement - Brownfield Enhancement

## User Story

As a **player**,
I want **an improved achievement screen with better visual design and navigation**,
So that **I can easily browse my progress, understand achievement requirements, and feel rewarded for unlocking them**.

## Story Context

**Existing System Integration:**

- Integrates with: `draw_achievements_screen()` in `src/ui_achievements.rs`
- Technology: Macroquad rendering, `AchievementManager` from `src/achievement.rs`
- Follows pattern: Category-based card grid layout
- Touch points: Achievement card rendering, scroll handling, category headers

**Current State:**
- Basic 4-column grid layout
- Category headers with simple styling
- Achievement cards with icon, name, description, tier badge
- Scroll support via `scroll_offset` parameter
- Dark theme with muted colors for locked achievements

## Proposed Improvements

### Visual Hierarchy Enhancements

| Element | Current | Proposed |
|---------|---------|----------|
| **Category Headers** | Plain text | Styled bar with icon and count |
| **Achievement Cards** | Flat cards | Subtle depth with hover-like active state |
| **Progress Indicator** | None | Overall completion bar at top |
| **Unlock State** | Muted colors | Clear locked/unlocked visual contrast |

### Layout Improvements

```
┌─────────────────────────────────────────────┐
│  ACHIEVEMENTS                               │
│  ████████████░░░░░░░░ 12/30 (40%)          │  ← Progress bar
├─────────────────────────────────────────────┤
│  🎯 Combat (4/8)                            │  ← Category with count
│  ┌────┐ ┌────┐ ┌────┐ ┌────┐               │
│  │ ✓  │ │ ✓  │ │ 🔒 │ │ 🔒 │               │
│  │Name│ │Name│ │ ?? │ │ ?? │               │
│  └────┘ └────┘ └────┘ └────┘               │
├─────────────────────────────────────────────┤
│  🏆 Mastery (2/6)                           │
│  ...                                         │
└─────────────────────────────────────────────┘
```

## Acceptance Criteria

**Functional Requirements:**

1. Progress bar showing total unlocked/total achievements with percentage
2. Category headers display unlock count (e.g., "Combat (4/8)")
3. Locked achievements show lock icon and "???" for hidden achievements
4. Unlocked achievements have distinct visual treatment (glow/border)

**Visual Requirements:**

5. Category sections have visual separation (divider or spacing)
6. Cards have subtle depth effect (shadow or gradient)
7. Tier badges (Bronze/Silver/Gold/Platinum) clearly visible
8. Color scheme consistent with game's space theme

**Integration Requirements:**

9. Existing scroll functionality preserved
10. ESC returns to ModeSelection (existing behavior)
11. Achievement unlock notifications unaffected
12. Performance acceptable (60 FPS with 30+ achievement cards)

**Quality Requirements:**

13. Readable on 800x600 minimum resolution
14. No visual glitches during scroll
15. Hidden achievements remain hidden until unlocked

## Technical Notes

- **Integration Approach:** Enhance existing rendering functions, add progress bar component
- **Existing Pattern Reference:** Category rendering at lines 102-128, card rendering at lines 175-290
- **Key Constraints:** Must work with current `AchievementManager` API

**Code Change Locations:**
```
src/ui_achievements.rs:11-300 (main rendering)
src/achievement.rs (may need get_total_count, get_unlocked_count methods)
```

**Implementation Steps:**

1. Add `get_stats()` method to AchievementManager if not exists
2. Create `draw_progress_bar()` helper function
3. Enhance `draw_category_header()` with unlock counts
4. Improve `draw_achievement_card()` with depth effects
5. Add visual dividers between categories
6. Test scroll behavior with enhanced visuals

## Risk and Compatibility Check

**Minimal Risk Assessment:**

- **Primary Risk:** Performance impact from additional visual effects
- **Mitigation:** Use simple gradients, limit shadow complexity
- **Rollback:** Revert to simple card style

**Compatibility Verification:**

- [x] No breaking changes to existing APIs
- [x] No database/save file changes
- [x] UI changes follow existing dark space theme
- [x] Performance impact monitored (target: 60 FPS)

## Definition of Done

- [ ] Progress bar displays at top of achievement screen
- [ ] Category headers show unlock counts
- [ ] Cards have improved visual depth
- [ ] Locked/unlocked states clearly distinguishable
- [ ] Scroll works correctly with new layout
- [ ] Performance verified (60 FPS on test machine)
- [ ] Manual test on multiple resolutions

## Effort Estimate

**Complexity:** Medium-High (visual polish work)
**Estimated Time:** 2-3 hours

## Optional Future Enhancements (Out of Scope)

- Filter by category/unlock status
- Sort options (by tier, by unlock date)
- Achievement detail popup on click
- Unlock animations
