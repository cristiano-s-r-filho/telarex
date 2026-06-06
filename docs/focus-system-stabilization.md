# Focus System Stabilization

This document outlines the root causes and technical solutions for the focus drift and "bounce" issues experienced during the TelaRex development phase.

## 1. Problem: The "Focus Bounce"
**Symptoms:** Users reported that focus would switch correctly (e.g., from Editor to File Explorer) but would immediately "bounce" back to the original target unsolicited.

**Root Causes:**
1.  **Double-Event Handling**: The event loop was processing both `KeyEventKind::Press` and `KeyEventKind::Release` events. Global shortcuts like `Ctrl+E` were resolving to `UIAction::SwitchFocus` twice in a single physical key press, effectively negating the switch.
2.  **KeyMapper Fallback**: The `KeyMapper` resolve logic had a fallback that defaulted any unknown component mode to `editor_normal`. This meant that even when the Explorer was focused, editor-specific movement keys (like `j`, `k`, `l`) were still active and potentially stealing focus or affecting editor state.
3.  **Aggressive Mouse Stealing**: The `Editor::handle_mouse` implementation was setting its internal `focused` state to `true` on any left-down event, regardless of whether the parent `EditorView` intended to shift focus.

## 2. Solutions Implemented

### A. Strict Event Gating
The `EditorView::handle_internal_event` loop was refactored to strictly gate action resolution. Global and component-specific actions are now only processed if `key.kind == KeyEventKind::Press`. This ensures that one physical key press results in exactly one state transition.

### B. Explicit Focus Hierarchy
Focus is now managed as a top-down "Push" model:
- `EditorView` maintains the `focused_child` (Explorer or Editor).
- Whenever `focused_child` changes, `update_focus_state()` is called.
- This method calls `LayoutTree::sync_focus(group_focused: bool)`.
- `sync_focus` pushes the boolean focus state into the `RefCell<bool>` of every individual `Editor` instance.
- Individual components are no longer allowed to set their own `focused` state; they must respect the state pushed by the parent view.

### C. Mouse Hit-Testing
Implemented absolute coordinate hit-testing in `EditorView`. By caching the `last_area` during the draw pass, the view can now accurately determine which component was clicked:
- Left 20% of the screen -> Focus Explorer.
- Remaining 80% -> Focus Editor.
This provides a natural and predictable way to switch focus that bypasses the keyboard event loop.

### D. Corrected KeyMapper Resolution
The `KeyMapper::resolve` method was refactored to treat component keymaps as exclusive. If a component is specified (e.g., "explorer"), only that component's map and the global map are checked. It no longer "bleeds" editor normal keys into other contexts.

## 3. Initialization
Updated `EditorView::new` to default `focused_child` to `FocusTarget::Explorer`, ensuring that users start in a navigation-ready state when opening a new project.
