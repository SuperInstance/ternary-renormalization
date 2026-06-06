# ternary-renormalization

**Renormalization group methods applied to ternary {-1, 0, +1} fields — coarse-graining, fixed points, critical exponents, and universality classes in discrete three-state systems.**

## Background

The renormalization group (RG) is one of the most powerful ideas in theoretical physics. Developed by Kenneth Wilson in the 1970s (earning him the 1982 Nobel Prize), RG explains how physical systems behave at different length scales. The core insight is deceptively simple: when you "zoom out" by averaging or coarse-graining local details, some properties remain unchanged while others vanish. The properties that persist under coarse-graining define the system's **universality class** — a deep structural invariant that transcends microscopic details.

In condensed matter physics, RG explains why water and iron undergo phase transitions with the same critical exponents — they share a universality class despite being utterly different materials. The Ising model, Potts model, and percolation all have well-characterized RG flows in the thermodynamic limit.

This crate applies RG to ternary {-1, 0, +1} fields. The question is: what happens when we coarse-grain a discrete three-state system? Each 2×2 block of ternary cells is replaced by a single ternary cell, and we track how observables (magnetization, energy, entropy) evolve as we repeatedly zoom out. In binary systems, the majority-rule RG of the Ising model produces exact results in two dimensions. In ternary, the presence of the zero state introduces a fundamentally different dynamic: ties become possible, and the zero state acts as an "absorbing" state that can dominate at large scales.

The mathematical framework draws on the **block spin renormalization** approach: partition the lattice into blocks, replace each block with an effective spin, and compute the renormalized coupling constants. For ternary systems, the coupling structure is richer than binary because there are three possible states and thus three distinct pair interactions (+1/+1, +1/0, +1/−1, etc.).

## How It Works

**`TernaryField`** — A 2D field of ternary values with observables:
- **`magnetization`**: The average value of all cells, computed as `(Σ cells × 3 / N)` clamped to {-1, 0, +1}. A fully +1 field gives magnetization = +1; a balanced field gives 0.
- **`energy`**: Nearest-neighbor Ising energy `E = -Σ s_i · s_j` over horizontal and vertical bonds. Aligned neighbors (e.g., +1/+1 or -1/-1) contribute -1; anti-aligned (+1/-1) contribute +1; mixed with zero contribute 0.
- **`entropy`**: Counts the number of distinct 3×3 patches, serving as a proxy for configurational entropy.

**Coarse-graining methods**:
- **`coarse_grain_majority`**: Each 2×2 block is replaced by its majority value. Ties default to 0. This is the standard block-spin RG rule.
- **`coarse_grain_sum`**: Each 2×2 block's values are summed and clamped to {-1, 0, +1}. This preserves more information about the block's net "charge."

**`RGFlow`** — Tracks observables across multiple coarse-graining steps:
- Records magnetization, energy, entropy, and resolution at each level.
- **`fixed_point`**: Detects when magnetization stabilizes (last two values are equal).
- **`is_critical`**: A system is at a critical point when entropy remains high across scales — the signature of scale-invariant structure (fractal patterns).

**`same_universality_class`** — Compares two RG flows: if they converge to the same fixed point, they belong to the same universality class.

**`critical_exponent`**: Computes a ternary approximation of the magnetization exponent β = log(|m₂|)/log(|m₁|), quantifying how quickly order disappears under coarse-graining.

### Design Decisions

- **Two coarse-graining rules**: Majority-rule preserves the dominant phase but loses information about ties; sum-rule preserves net magnetization but can amplify noise. Having both lets researchers study how the RG flow depends on the coarse-graining scheme.
- **No random number generation**: The coarse-graining is deterministic, making results reproducible. In statistical mechanics, one typically averages over disorder realizations; here, we study single configurations.
- **`#![no_std]` compatible**: Suitable for deployment on ternary hardware.

## Experimental Results

All 13 tests pass. Key observations:

- **Uniform field +1**: A 4×4 field of all +1 has magnetization = +1 and negative energy (all bonds aligned). After coarse-graining by majority, the 2×2 result is still all +1. The RG flow reaches a fixed point at magnetization = +1 in 2+ levels — the ordered phase is stable under RG.
- **Majority coarse-graining of a uniform block**: A 4×4 field where only the top-left 2×2 block is +1, rest is 0. The coarse-grained 2×2 field has `cg.get(0,0) = +1` and the other three cells = 0. Majority rule correctly identifies the ordered region.
- **Sum coarse-graining**: The same +1 block gives sum = 4, clamped to +1. For a block with two +1 and two -1, the sum is 0 — the zero state absorbs balanced configurations.
- **Entropy of uniform field**: An all-+1 6×6 field has entropy = 1 (only one distinct 3×3 patch). A checkerboard with period-3 has entropy ≥ 1 with multiple distinct patches.
- **Universality class matching**: Two identical 8×8 fields of all +1 produce RG flows with the same fixed point (+1), confirming `same_universality_class` returns true.
- **Critical exponent**: For a uniform +1 field, the critical exponent β is in {-1, 0, +1} — the ternary quantization limits the resolution of this measurement.

## Impact

Renormalization in ternary matters because:

1. **Network pruning decisions**: When coarse-graining a ternary neural network's weight matrix, RG tells you which features survive at larger scales. Features that vanish under RG are noise; features that persist are structural.
2. **Multiscale analysis of ternary data**: Satellite imagery quantized to ternary, sensor arrays with three-state outputs — RG provides a principled framework for analyzing structure across scales.
3. **Theoretical foundation**: This is the first (to our knowledge) implementation of block-spin RG for Z₃-valued fields, providing a computational laboratory for studying ternary critical phenomena.

## Use Cases

1. **Ternary neural network pruning analysis**: Apply RG to the weight matrices of a trained ternary network. If coarse-graining preserves the network's output, the pruned (coarse-grained) weights define a smaller, equivalent network — a principled pruning criterion.

2. **Multiscale image analysis in ternary**: Quantize a grayscale image to ternary {-1, 0, +1} (below threshold, at threshold, above threshold). Apply RG to detect whether features persist across scales — a measure of self-similarity useful for texture classification.

3. **Criticality detection in agent populations**: Model a population of agents as a ternary field (agent state = -1/0/+1). Track whether the population is at a critical point (entropy stays high under RG), which predicts sensitivity to perturbations.

4. **Material phase diagram exploration**: Use RG to map the phase diagram of ternary lattice models. Vary the initial conditions and track which fixed points the system flows to — a computational substitute for expensive Monte Carlo simulations.

5. **Data compression**: Coarse-graining is a form of lossy compression. RG tells you how much information is lost at each scale, enabling rate-distortion analysis for ternary data streams.

## Open Questions

1. **Ternary critical exponents**: In the binary Ising model, the 2D critical exponent β ≈ 1/8 is known exactly. What are the critical exponents for ternary (Z₃) systems? Our coarse-graining approach gives only {-1, 0, +1}-valued approximations, which are too coarse for accurate exponent estimation.

2. **Optimal coarse-graining rule**: Majority-rule and sum-rule are natural choices, but are they optimal? There may be coarse-graining schemes that preserve more information about the original field, especially near criticality.

3. **Finite-size effects**: Our tests use 4×4 to 8×8 fields, which are tiny by physics standards. How do the RG flows change as the system size increases toward the thermodynamic limit?

## Connection to Oxide Stack

In the five-layer architecture, `ternary-renormalization` operates at the **flux-core** level as a multiscale analysis tool. Its RG flow analysis can inform the **pincher** layer's decision about when to stop iterating (convergence detection via fixed points). At the **cuda-oxide** level, coarse-graining is an embarrassingly parallel operation (each 2×2 block is independent), making it an ideal candidate for GPU acceleration. The entropy metric feeds directly into **cudaclaw**'s reporting infrastructure, providing users with a quantitative measure of system complexity.
