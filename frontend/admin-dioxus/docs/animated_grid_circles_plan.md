# Animated Grid Circles – Implementation Plan

Goal: Replace keyframe-based single circle with a pool of independently animated circles that move one grid cell at a time using CSS transitions. Each circle owns its movement loop and respawn logic, reacts to grid changes, and never leaves bounds.

## Objectives
- Render many circles via a parent `AnimatedGridCircles` that holds `Vec<GlobalSignal<GridCircle>>` and maps each to `AnimatedGridCircle`.
- No CSS keyframes. Use `transition` on `transform` and update inline `style` per step; chain steps with `ontransitionend`.
- Circles spawn on an edge and travel toward the opposite direction (`travel_dir = spawn_edge.reverse()`).
- Move exactly one grid cell per transition. At each cell boundary: 20% chance to insert a vertical/horizontal “side step” while preserving the overall goal direction.
- Stay within bounds; clamp indices and respawn when exiting the playable area.
- Each circle handles its own `ontransitionend`, next move decision, and respawn.

## Data Model
- `enum Direction { Left, Right, Up, Down }`
  - `fn reverse(&self) -> Direction`
  - `fn axis(&self) -> Axis` where `Axis` = `Horizontal | Vertical`
  - `fn delta(&self) -> (i32, i32)`  // column/row delta for one step

- `enum SpawnEdge { Left, Right, Top, Bottom }`
  - `impl From<SpawnEdge> for Direction` → returns travel_dir as reverse(edge)

- `struct GridCircle {
    id: u64,
    col: i32,
    row: i32,
    travel_dir: Direction,
    moving: bool,                       // suppress overlapping transitions
    respawning: bool,                   // render without transition when snapping
    step_ms: u32,                       // duration per cell
    diameter_px: f64,
    spawn_edge: SpawnEdge,
    alive: bool,
}`

- `type CircleSig = GlobalSignal<GridCircle>`

## Integration Points
- Grid context: `GridData { vertical_lines, horizontal_lines, dimensions }` already exists (`src/components/animated_grid/provider.rs`).
  - Compute `cols = vertical_lines.len() - 1`, `rows = horizontal_lines.len() - 1`.
  - Require at least two lines per axis before spawning (`vertical_lines.len() >= 2 && horizontal_lines.len() >= 2`).
  - Compute `cell_w = vertical_lines[1] - vertical_lines[0]` and `cell_h = horizontal_lines[1] - horizontal_lines[0]` (skip if non-positive). With square grids they should be approximately equal.
  - Convert `(col,row)` → `(x,y)` pixels:
    - `x = vertical_lines[col as usize]`
    - `y = horizontal_lines[row as usize]`
    - Center the circle: add half of cell size minus half of diameter.

## Rendering Structure
- Parent: `AnimatedGridCircles { count: usize }`
  - On mount or when grid is ready: create `count` circles with random edges and positions along that edge; set `travel_dir = edge.reverse()`.
  - Keep `Vec<CircleSig>` in component state and render: `{circles.iter().map(|c| rsx!(AnimatedGridCircle { circle: c.clone() }))}`
  - On grid change (dimensions/lines changed): notify circles to snap to nearest valid index and continue. Easiest: mark `moving = false` and force reflow to new `(x,y)`, then resume next step.

- Child: `AnimatedGridCircle { circle: CircleSig }`
  - Renders a small absolutely-positioned div:
    - `class: "absolute pointer-events-none will-change-transform"` plus `transition-transform` only when `!respawning` to avoid flicker on snap
    - `style: format!("transform: translate({x}px, {y}px);{} width: {}px; height: {}px; border-radius: 9999px;",
        if respawning { "" } else { format!(" transition-duration: {}ms;", step_ms) }, d, d)`
  - Event: `ontransitionend: move |_| circle_step(circle.clone(), grid_ctx.clone())`
  - On first mount: kick off initial step by calling `circle_step` if not moving.

## Movement Algorithm (per circle)
1. Preconditions: require grid lines present (non-empty). If grid isn’t ready, delay/retry.
2. If `moving == true`, ignore requests to step.
3. Determine next move (evaluate every step):
   - Primary move is one cell in `travel_dir` (`delta_col, delta_row`).
   - Roll a 20% chance each step to choose a perpendicular side step while keeping overall direction:
     - If `travel_dir` is `Left` or `Right`: consider `Up` or `Down` by 1.
     - If `travel_dir` is `Up` or `Down`: consider `Left` or `Right` by 1.
     - If the candidate side step would exit bounds, ignore it and use the primary move.
   - Only a single cell is moved per transition; no queued multi-step moves.
4. Bounds check: ensure `0 <= next_col < cols` and `0 <= next_row < rows`. If the move would exit bounds, mark `alive = false` and respawn.
5. Update state: set `moving = true`, update `col/row` to next indices, compute new `(x,y)` pixels, and update style. The CSS transition will animate from current transform to the new transform.
6. `ontransitionend` handler:
   - Set `moving = false` and call `circle_step` directly to choose and perform the next one-cell move.

## Respawn Logic
- When a circle attempts to move out of bounds:
  - Pick a new `spawn_edge` (random among 4), set `(col,row)` to a valid index on that edge (random along axis), set `travel_dir = edge.reverse()`.
  - Reset `moving = false`, set `respawning = true`, snap transform immediately to spawn position, then set `respawning = false` so subsequent moves transition normally.
  - After re-positioning, trigger the first step in the new direction.

## Randomness Strategy
- Abstraction: `trait Rng { fn chance(percent: u8) -> bool; fn range(max: i32) -> i32; }`
- Web implementation: use `js_sys::Math::random()` to avoid extra deps, or add `getrandom`/`rand` with `wasm_js` feature.
- Deterministic fallback: simple xorshift/LCG seeded by `id` if needed.

## Durations and Easing
- Use `transition: transform linear;` with `transition-duration: step_ms ms` (e.g., 300–600ms). Keep per-circle `step_ms` to add variety.
- Keep sizes small (e.g., 6–8px) and include a soft shadow to match current visuals.

## API Sketch
- `AnimatedGridCircles { count: usize }`
- `AnimatedGridCircle { circle: GlobalSignal<GridCircle> }`
- Helpers:
  - `fn spawn_circle(id: u64, grid: &GridData) -> GridCircle`
  - `fn step_once(circle: &mut GridCircle, grid: &GridData, rng: &impl Rng) -> Option<(col,row)>` (returns next target or `None` for respawn)
  - `fn indices_to_px(col,row, grid: &GridData, d: f64) -> (f64,f64)`

## Edge Cases & Safeguards
- If grid changes mid-transition, finish current step then snap to nearest valid indices and continue; if indices out of range, respawn.
- Prevent back-to-back `ontransitionend` loops by checking `moving` and ensuring target changed.
- Clamp side steps that would leave bounds; ignore invalid side steps.

## Work Plan (progress as of current branch)
1. ✅ Types and utilities  
   `Direction`, `SpawnEdge`, `GridCircle`, and RNG helpers now live in `src/components/animated_grid/circle.rs`, covering the foundational data model.
2. ✅ Parent container  
   `AnimatedGridCircles` instantiates the pool, clamps the count, and drives each circle’s loop, satisfying the parent orchestration requirement.
3. ✅ Child component  
   `AnimatedGridCircle` renders absolutely-positioned circles, applies per-step transforms, and uses `ontransitionend` to trigger the next move.
4. ✅ Grid integration  
   Movement now derives from `GridData` cell metrics and pixel conversions, ensuring circles stay aligned with the grid layout.
5. ✅ Respawn + randomness  
   Edge spawning, 20 % perpendicular side steps, and per-circle duration/size variance are implemented with bounds enforcement.
6. ⏳ Resize behavior  
   Still need to re-sync or respawn circles automatically when grid lines change; the current effect only runs on mount.
7. ⏳ Polish  
   Shadows and motion basics exist, but there’s no per-circle signal isolation, feature flagging, or extra styling tweaks yet.

## Capacity
- Default to a modest circle count (e.g., 12–24) to ensure smooth rendering when using `GlobalSignal` updates per step. Make the count configurable.

## Files to Add/Change (proposal)
- `src/components/animated_grid/circles.rs` (parent + child components)
- `src/components/animated_grid/types.rs` (Direction, SpawnEdge, GridCircle)
- Integrate from `src/components/animated_grid/mod.rs`
