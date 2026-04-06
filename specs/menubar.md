# Menubar UX Spec

## Status

- **Current target:** panel disclosure (Variant 3, Strict System)
- **Legacy reference only:** true native NSMenu behavior is documented and pinned via git baseline

### V1 baseline reference (git)

The latest known V1-native menubar baseline is pinned to git commit:

- full: `d17db4472ca3006aae86716a4dccf8e88b190417`
- short: `d17db44`
- date: `2026-03-24T00:07:16+02:00`
- message: `add checks rollup to ado-pr feed with CI-only filtering and expired build detection`

Use this commit as the source-of-truth snapshot for "pure native menubar" behavior before V2 panel implementation.

This document extends `specs/main.md` with menubar UX decisions and implementation constraints.

If any statement here conflicts with `specs/main.md`, treat `specs/main.md` as canonical until both docs are updated together.

---

## 1) Goals and Non-Goals

### Goals

1. Ship and maintain a single menubar UX: **panel disclosure (Variant 3, Strict System)**.
2. Keep the UI deeply macOS-native in feel while enabling richer behavior than NSMenu.
3. Keep Feed/Activity/Field data semantics stable while evolving only the rendering layer.

### Non-Goals

1. Custom AppKit `NSView` menu-row rendering inside `NSMenu`.
2. Replacing feed runtime, polling model, or `FeedSnapshot` schema for UX reasons.
3. Building a full standalone dashboard window.

---

## 2) Architecture Seam (Must Stay Stable)

The core seam between feed runtime and UI is the **snapshot data contract**:

- Source: `FeedSnapshotCache`
- Shape: list of `FeedSnapshot` containing `Activity` and `Field`
- Existing command: `list_feeds` (`src-tauri/src/command.rs`)

Rule: rendering may evolve, but snapshot semantics and lifecycle stay consistent.

---

## 3) Legacy Native Reference (Not a maintained runtime mode)

### 3.1 Definition

The legacy native behavior is the `NSMenu` implementation in `src-tauri/src/tray.rs`, pinned by the baseline commit in this file.

It uses Tauri native menu primitives only:

- `MenuItem`
- `Submenu`
- `IconMenuItem`
- `PredefinedMenuItem::separator`

### 3.2 Legacy behavior snapshot

1. Top level is grouped by **Feed**.
2. Feed heading rows are non-interactive.
3. Each **Activity** appears as a submenu title with status symbol prefix.
4. Opening an Activity submenu reveals **Field** rows (`label: value`).
5. Activity open action appears as `Open` within submenu.
6. Retained Activities use hollow dot (`◦`) and render after active Activities.
7. Feed-level errors render at feed section level.
8. Bottom global actions: `Refresh feeds`, `Quit Cortado`.

### 3.3 Status Kind Precedence Contract

Activity status kind precedence (see `specs/status.md` for the full model):

1. `attention-negative`
2. `waiting`
3. `running`
4. `attention-positive`
5. fallback `idle`

This precedence must remain identical in current and future panel implementations.

### 3.4 Native constraints (reason for moving away)

1. Non-clickable Feed text appears disabled/gray.
2. No per-Activity row color coding at top level.
3. Rich inline expansion is not possible with current primitive set.

---

## 4) Technical Debt Notes (from native baseline)

This section tracks lessons from the native baseline that should inform panel implementation quality.

| ID | Debt | Impact | Required mitigation |
|---|---|---|---|
| TD-01 | URL/open behavior could diverge across UI refactors | User-visible inconsistency | Centralize open-action command/utility |
| TD-02 | Feed field selection logic is tray-specific (`fields_for_activity_menu`) | Behavior drift with new feed types | Define panel field-priority rules explicitly and test them |
| TD-03 | Full menu rebuild model (`set_menu`) informed old perf assumptions | Misleading perf baseline for panel work | Define panel perf budgets from panel behavior, not menu rebuild behavior |

### Native Recovery note

Returning to native NSMenu is explicitly considered a future development effort, not a runtime toggle.

---

## 5) Current Contract -- Panel Disclosure (Variant 3, Strict System)

Current menubar direction: **inline expandable disclosure panel** (Control Center / Wi-Fi style), using the **Strict System** sub-variant.

### 5.1 Principles

1. **Native-first visual language**
   - SF Pro typography scale
   - restrained spacing and contrast
   - subtle vibrancy-like surface, minimal ornament

2. **Glance first, details on demand**
   - compact Activity row shows status + title + key signal
   - row expands inline to reveal all Fields

3. **Color as semantic signal, not decoration**
   - color reserved for status communication
   - no gratuitous gradients/glow styles

4. **Keyboard and pointer parity**
   - pointer click toggles disclosure
   - keyboard navigation and expand/collapse supported

5. **Single maintained UI path**
   - one production menubar implementation
   - no maintained runtime dual-mode UX

### 5.2 High-Level Interaction Decisions

1. **Panel behavior**
   - Menubar-attached transient panel
   - closes on focus loss / app switch / space change

2. **Disclosure behavior**
   - Activity row toggles inline detail region
   - details contain full Field list (`label: value`) and `Open` action

3. **Grouping model**
   - Feed is top-level section
   - Activity is row inside Feed
   - Field rows only appear in expanded Activity region

4. **Status presentation**
   - status dot + optional compact status chip on row
   - retained Activities use hollow marker treatment

5. **Error model**
   - feed-level errors displayed inline per Feed section
   - does not block other Feeds from rendering

### 5.3 Why this variant

Compared to hover submenus or card-heavy alternatives, Variant 3 best balances:

- native familiarity (Wi-Fi/Bluetooth disclosure pattern)
- high information density
- lower cognitive load than multi-panel submenu navigation
- accessible keyboard flow

### 5.4 V2 Non-Goals

1. No custom charting or dashboard modules in panel.
2. No multi-step workflow inside panel.
3. No persistence of disclosure state across app restarts (initially).

---

## 6) Rollback Philosophy

Rollback to native is possible through standard git + development workflow using the pinned baseline commit.

### 6.1 Invariants

1. Same Feed/Activity/Field snapshot contract across menubar revisions.
2. Same status kind precedence across menubar revisions.
3. Same retained ordering rule across menubar revisions.
4. Same open-action semantics across menubar revisions.

No schema migration, feed reconfiguration, or data conversion should be required for rollback work.

---

## 7) Acceptance Criteria

### 7.1 Panel implementation

- [ ] Panel opens from menubar and closes on focus loss.
- [ ] Feed headers are readable/non-disabled in appearance.
- [ ] Activity rows are color-coded by status kind at top level.
- [ ] Inline expand reveals full Field list.
- [ ] Keyboard navigation and expand/collapse work.
- [ ] Error and empty states are clear and consistent.

### 7.2 Consistency across revisions

- [ ] Status kind precedence matches exactly.
- [ ] Retained behavior matches exactly.
- [ ] Open target selection rules match exactly.

---

## 8) Validation Checklist (Release Gate)

Run before any release touching menubar UX:

1. Validate panel mode end-to-end.
2. Confirm no Feed runtime behavior changed unintentionally.
3. Confirm terminology remains Feed / Activity / Field in UX copy and docs.

---

## 9) Decision Log

- **Accepted:** Menubar uses a single maintained UI path (panel disclosure).
- **Accepted:** Variant 3 visual sub-variant is **Strict System**.
- **Accepted:** Strict System keeps native-style row hover highlight for Activity rows.
- **Accepted:** Native NSMenu behavior is preserved as git history reference, not as a maintained runtime mode.
