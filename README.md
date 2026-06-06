# ternary-renormalization

The renormalization group on ternary fields — coarse-graining reveals fixed points, critical exponents, and universality.

## Why This Exists

The renormalization group (RG) is one of physics' deepest ideas: when you zoom out from a system, certain properties survive and others wash away. Zoom out enough and every system falls into a **universality class** — systems that look completely different at the microscopic level share the same large-scale behavior. Water-liquid transitions and magnet transitions obey the same mathematics. This is not a metaphor; it's a theorem.

This crate implements RG flow on ternary fields `{-1, 0, +1}`. You start with a fine-grained ternary field, apply majority-rule or sum-rule coarse-graining (reduce resolution by 2×), and measure observables at each scale: magnetization, energy, and entropy. The sequence of measurements is the RG flow trajectory. Fixed points (where observables stop changing) identify phase behavior. Critical exponents (how observables vanish near criticality) classify universality.

`#![no_std]` compatible — pure algebra, no system dependencies.

## Architecture

```
TernaryField (width × height, cells: Vec<i8>)
    │
    ├── Observables
    │   ├── magnetization() → i8       (average cell value)
    │   ├── energy() → i32             (disagreeing neighbor pairs)
    │   └── entropy() → usize          (distinct 3×3 patches)
    │
    ├── Coarse-graining
    │   ├── coarse_grain_majority() → TernaryField  (2×2 blocks → majority vote)
    │   └── coarse_grain_sum() → TernaryField       (2×2 blocks → sum, clamped)
    │
    ▼
RGFlow (tracks observables across scales)
    ├── magnetization_history: Vec<i8>
    ├── energy_history: Vec<i32>
    ├── entropy_history: Vec<usize>
    ├── scales: Vec<usize>            (resolution at each level)
    │
    ├── fixed_point() → Option<i8>    (magnetization stops changing)
    ├── is_critical() → bool          (entropy stays high across scales)
    │
    ▼
Universality Classification
    ├── same_universality_class(flow_a, flow_b) → bool
    └── critical_exponent(flow) → i8  (magnetization decay rate)
```

**Key types:**

- **`TernaryField`** — a 2D grid of `i8` values in `{-1, 0, +1}`. Supports two coarse-graining strategies and three observable measurements.
- **`RGFlow`** — the result of running RG flow on a field. Tracks how magnetization, energy, and entropy evolve across coarse-graining levels.
- **`same_universality_class()`** — compares two RG flows. Systems in the same universality class converge to the same fixed point.
- **`critical_exponent()`** — measures how magnetization vanishes under coarse-graining. Related to the β exponent in critical phenomena.

## Usage

```rust
use ternary_renormalization::{TernaryField, RGFlow, same_universality_class, critical_exponent};

// Create a uniform field (ordered phase)
let mut ordered = TernaryField::new(16, 16);
for y in 0..16 {
    for x in 0..16 {
        ordered.set(x, y, 1);
    }
}

// Run RG flow: coarse-grain repeatedly
let flow = RGFlow::run(&ordered, 4); // up to 4 levels of coarse-graining
assert_eq!(flow.fixed_point(), Some(1)); // magnetization stays at +1
// Ordered phase: fixed point at full magnetization

// Create a mixed field (near critical)
let mut mixed = TernaryField::new(16, 16);
for y in 0..16 {
    for x in 0..16 {
        mixed.set(x, y, if (x + y) % 2 == 0 { 1 } else { -1 });
    }
}
let flow_mixed = RGFlow::run(&mixed, 4);

// Check if the system is at a critical point
// (entropy stays high across scales = scale-invariant fluctuations)
println!("Critical? {}", flow_mixed.is_critical());

// Compare two systems: same universality class?
let flow1 = RGFlow::run(&ordered, 4);
let flow2 = RGFlow::run(&ordered, 4);
assert!(same_universality_class(&flow1, &flow2)); // identical systems

// Critical exponent: how fast does magnetization decay?
let beta = critical_exponent(&flow1);
println!("Critical exponent: {}", beta);

// Coarse-graining strategies
let cg_majority = ordered.coarse_grain_majority(); // 2×2 blocks → majority vote
let cg_sum = ordered.coarse_grain_sum();           // 2×2 blocks → sum, clamped

// Observables
let m = ordered.magnetization();  // net alignment
let e = ordered.energy();         // neighbor agreement (negative = aligned)
let s = ordered.entropy();        // number of distinct local patterns

// Entropy of a uniform field
assert_eq!(ordered.entropy(), 1); // only one distinct 3×3 pattern
```

## API Reference

### `TernaryField`

| Method | Description |
|--------|-------------|
| `TernaryField::new(width, height)` | Create field initialized to 0 |
| `.get(x, y)` / `.set(x, y, v)` | Cell access (values clamped to `{-1, 0, +1}`) |
| `.coarse_grain_majority()` | Reduce resolution 2×. Each 2×2 block → majority value. Ties → 0. |
| `.coarse_grain_sum()` | Reduce resolution 2×. Each 2×2 block → sum, clamped to ternary. |
| `.magnetization()` | Average cell value, quantized to `i8` |
| `.energy()` | Count of disagreeing neighbor pairs (Ising energy) |
| `.entropy()` | Number of distinct 3×3 patches |

### `RGFlow`

| Method | Description |
|--------|-------------|
| `RGFlow::run(initial, max_levels)` | Run RG flow on a field. Coarse-grain up to `max_levels` times, stopping when resolution < 4. |
| `.fixed_point()` | Magnetization at last two levels are equal → fixed point found |
| `.is_critical()` | Minimum entropy > half of maximum entropy (scale-invariant) |

Fields: `magnetization_history`, `energy_history`, `entropy_history`, `scales`

### Free Functions

| Function | Description |
|----------|-------------|
| `same_universality_class(flow_a, flow_b)` | True if both have the same fixed point |
| `critical_exponent(flow)` | Ratio of magnetization change between last two scales, clamped to ternary |

## The Deeper Idea

The renormalization group answers a deep question: **which microscopic details matter?** The answer is: most don't. When you coarse-grain a system — averaging over small-scale fluctuations — only a few parameters survive. These are the **relevant operators** in RG language. Everything else is washed away by the coarse-graining.

In the ternary framework, this plays out as follows. Start with an arbitrary ternary field. Apply majority-rule coarse-graining. At each level, measure magnetization, energy, and entropy. The trajectory of these observables is the RG flow. Three things can happen:

1. **Flow to ordered fixed point** (magnetization → ±1): the system is in a uniform phase. All fluctuations are suppressed. The microscopic details don't matter — the system forgets its initial conditions.

2. **Flow to disordered fixed point** (magnetization → 0): the system is in a random phase. All correlations decay. Again, initial conditions are forgotten.

3. **Critical flow** (entropy stays high across scales): the system is at a phase transition. Fluctuations exist at *every* scale — the system is scale-invariant. This is where the interesting physics lives.

The ternary simplification — three states instead of a continuum — makes the RG flow computationally trivial but conceptually complete. You lose the ability to study continuous phase transitions, but you gain clarity: the fixed points are exactly `{−1, 0, +1}`, the critical exponent is quantized, and universality classes are determined by which fixed point the flow converges to.

Universality is the punchline. Two ternary fields with completely different initial conditions — one generated by a reaction-diffusion process, another by a percolation process — can flow to the same fixed point. This means they share the same large-scale behavior despite having nothing in common at the microscopic level. That's the power of the renormalization group: it separates the universal from the particular.

## Related Crates

- **`ternary-morphogenesis`** — reaction-diffusion on ternary grids, whose patterns are analyzed by this crate
- **`ternary-percolate`** — percolation on ternary grids, another source of fields for RG analysis
- **`conservation-spectral-topology-rs`** — spectral analysis of graph structure, complementing RG's real-space analysis
