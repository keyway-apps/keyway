# Workspace items and navigation

`ui::NavStackState` owns the current page, back entries, forward entries and
transitions. Pane keeps an opening-order cache of `Box<dyn ItemHandle>` for
entity reuse and Item-specific focus/bar integration. There is no separate
navigation history, active index, activation history or timestamp counter.

`Pane::add_item(item, focus_item, activate, window, cx)` reuses an entity by ID
without reordering the cache. New items append to the cache. With `activate`
enabled, `activate_item` calls `push(index, cx)` to open the cached entity's view.
Otherwise only the cache changes, including when opening the first item.
Repeated activation of the current page adds no navigation entry; A -> B -> A
remains three visits. `focus_item` moves keyboard focus only with activation.
`active_item(cx)` resolves the stack's current view through the cache.

`Workspace::go_back` calls `pop_without_history(NavMotion::Animated, cx)`, then
refreshes bars and focus without pushing the destination again. It retains the
root and returns `false` if no previous page exists. The popped navigation entry
is discarded without creating a forward entry; its Item remains cached for
reuse, so returning does not call `on_removed`.

`Pane::remove_item` deletes a cached Item and every visit to it in the stack,
including forward entries and an outgoing animation reference. It calls
`ItemHandle::on_removed`, forwarded to `Item::on_removed`, once per removal from
the cache. Removing an unknown ID is a no-op. Removing the active page reveals
the previous surviving page immediately; without one, the pane displays empty.
The last Item can be removed. Removal never activates another cached neighbour.
Focus follows removal when requested or when the removed Item held focus.

The cache includes the current page and holds at most ten different entities.
New opens evict the oldest inactive Item through `remove_item`, including the
removal hook and pruning of its history. Opening order is independent of
navigation order. Removing a cached entity releases the pane's ownership;
external handles and previously rendered frames may extend its lifetime.

## Local Nav Stack

`crates/ui/src/components/nav_stack.rs` is adapted from GPUI Base 0.9.1 at
revision `39ab849a93b5f15507b969164d49891db8d5b470`. The source records its
provenance and the adjacent directory includes the Apache-2.0 license. The local
copy reuses GPUI's `History`, `Presence`, `Transition`, `NavMotion` and
`NavOperation`. No dependency source or Cargo patch is changed. Import the local
`NavStack`, `NavStackState`, `NavStackEvent` and `NavPage` through `ui`.

- `pop_without_history(motion, cx) -> Option<AnyView>` removes the current visit
  without adding a forward entry. It preserves the root, older visits and any
  existing forward branch. An animated pop retains the outgoing view only until
  the exit completes; an immediate pop keeps no outgoing reference.
- `remove(entity_id, cx) -> bool` removes every matching visit from both history
  branches in place and collapses adjacent visits to the same entity. It can
  delete a middle, current, root or forward destination. Without a surviving
  back entry, the forward branch is cleared. It also releases a matching
  outgoing transition, even if no history entry remains. It returns whether
  any stack reference changed and emits one `NavStackEvent::Removed` on change.
- Middle/forward pruning preserves the current view and unrelated transitions,
  adjusting outgoing indices without restarting their animations. Removing the
  current view switches immediately and cancels its obsolete transition.

Ordinary `pop` still records forward history; `forward`, `pop_to_root`, `push`,
`replace` and `clear` remain available. The history limit remains 1000 back
entries. Removal scans history in place in O(n), without cloning/rebuilding the
stack or emitting a sequence of clear/push/pop events. Destructive pop is O(1).

Pane exposes no `go_back` or `set_active_item` helper. `Workspace::add_item`
retains its existing placeholder implementation; callers open Items through
`Workspace::active_pane()` and `Pane::add_item`.
