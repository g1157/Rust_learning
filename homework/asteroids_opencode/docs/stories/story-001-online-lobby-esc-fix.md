# Story 001: Online Lobby ESC Exit Bug Fix - Brownfield Addition

## User Story

As a **player**,
I want **to exit the online lobby by pressing ESC and return to the main menu**,
So that **I can navigate back when the server is unavailable or I change my mind**.

## Story Context

**Existing System Integration:**

- Integrates with: `GameState` state machine in `src/game_state.rs`
- Technology: Rust + Macroquad input handling
- Follows pattern: Other ESC handlers in `OnlineWaiting` state (line 1656)
- Touch points: `src/main.rs:1631-1640` (OnlineLobby ESC handler)

**Current Bug:**
ESC key in `OnlineLobby` state sets `state = GameState::OnlineLobby { nickname_input: false }` instead of returning to `GameState::ModeSelection`.

## Acceptance Criteria

**Functional Requirements:**

1. Pressing ESC in OnlineLobby state returns to ModeSelection with Online mode selected
2. Network client properly disconnects/leaves queue before state transition
3. User can re-enter online mode after exiting

**Integration Requirements:**

4. Existing OnlineWaiting ESC behavior unchanged
5. Network client state is properly cleaned up
6. No memory leaks from abandoned connections

**Quality Requirements:**

7. Manual testing confirms ESC exits to main menu
8. No regression in other ESC handlers

## Technical Notes

- **Integration Approach:** Change state transition target from `OnlineLobby` to `ModeSelection`
- **Existing Pattern Reference:** See `OnlineWaiting` ESC handler at line 1656 which correctly returns to `OnlineLobby`
- **Key Constraints:** Must call `network_client.disconnect()` or send `LeaveQueue` before exit

**Code Change Location:**
```
src/main.rs:1631-1640
```

**Proposed Fix:**
```rust
// ESC 返回主菜单
if input_state.is_key_pressed(KeyCode::Escape) {
    network_client.disconnect();  // Ensure clean disconnect
    state = GameState::ModeSelection {
        selection: GameMode::Online,
    };
    next_frame().await;
    continue;
}
```

## Risk and Compatibility Check

**Minimal Risk Assessment:**

- **Primary Risk:** Network connection left hanging if not properly closed
- **Mitigation:** Call disconnect() before state change
- **Rollback:** Revert single line change

**Compatibility Verification:**

- [x] No breaking changes to existing APIs
- [x] No database changes
- [x] UI changes follow existing design patterns
- [x] Performance impact is negligible

## Definition of Done

- [ ] ESC in OnlineLobby returns to ModeSelection
- [ ] Network client properly disconnects
- [ ] Manual test: Enter online mode → ESC → returns to menu
- [ ] No regression in OnlineWaiting ESC behavior
- [ ] Code follows existing patterns

## Effort Estimate

**Complexity:** Low (single state transition fix)
**Estimated Time:** < 30 minutes
