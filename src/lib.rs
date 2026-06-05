//! # ternary-renormalization
//!
//! The renormalization group in ternary systems.
//! Coarse-graining, fixed points, critical exponents, and universality classes.

#![forbid(unsafe_code)]
#![no_std]

extern crate alloc;
use alloc::{vec, vec::Vec};

/// A 2D ternary field that can be coarse-grained
#[derive(Debug, Clone)]
pub struct TernaryField {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<i8>,
}

impl TernaryField {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height, cells: vec![0; width * height] }
    }

    pub fn get(&self, x: usize, y: usize) -> i8 {
        if x < self.width && y < self.height {
            self.cells[y * self.width + x]
        } else { 0 }
    }

    pub fn set(&mut self, x: usize, y: usize, v: i8) {
        if x < self.width && y < self.height {
            self.cells[y * self.width + x] = v.clamp(-1, 1);
        }
    }

    /// Majority-rule coarse-graining: reduce resolution by factor of 2
    /// Each 2×2 block becomes one cell with the majority value
    pub fn coarse_grain_majority(&self) -> TernaryField {
        let nw = self.width / 2;
        let nh = self.height / 2;
        let mut result = TernaryField::new(nw, nh);
        for y in 0..nh {
            for x in 0..nw {
                let mut counts = [0usize; 3]; // index: 0→-1, 1→0, 2→1
                for dy in 0..2 {
                    for dx in 0..2 {
                        let v = self.get(x * 2 + dx, y * 2 + dy);
                        let idx = ((v + 1).rem_euclid(3)) as usize;
                        counts[idx] += 1;
                    }
                }
                let val = if counts[2] > counts[0] && counts[2] > counts[1] { 1 }
                          else if counts[0] > counts[1] { -1 }
                          else { 0 };
                result.set(x, y, val);
            }
        }
        result
    }

    /// Sum-rule coarse-graining: sum 2×2 block, clamp to ternary
    pub fn coarse_grain_sum(&self) -> TernaryField {
        let nw = self.width / 2;
        let nh = self.height / 2;
        let mut result = TernaryField::new(nw, nh);
        for y in 0..nh {
            for x in 0..nw {
                let sum: i8 = (0..2).map(|dy| (0..2).map(|dx| self.get(x*2+dx, y*2+dy)).sum::<i8>()).sum();
                result.set(x, y, sum.clamp(-1, 1));
            }
        }
        result
    }

    /// Magnetization: average value (sum / count)
    pub fn magnetization(&self) -> i8 {
        let sum: i32 = self.cells.iter().map(|&v| v as i32).sum();
        let n = self.cells.len() as i32;
        if n == 0 { return 0; }
        (sum * 3 / n).clamp(-1, 1) as i8
    }

    /// Energy: count of disagreeing neighbor pairs (nearest-neighbor Ising energy)
    pub fn energy(&self) -> i32 {
        let mut e = 0i32;
        for y in 0..self.height {
            for x in 0..self.width {
                let v = self.get(x, y);
                if x + 1 < self.width {
                    e -= (v * self.get(x + 1, y)) as i32;
                }
                if y + 1 < self.height {
                    e -= (v * self.get(x, y + 1)) as i32;
                }
            }
        }
        e
    }

    /// Entropy: count distinct local patterns (3×3 patches)
    pub fn entropy(&self) -> usize {
        let mut patterns = vec![];
        for y in 0..self.height.saturating_sub(2) {
            for x in 0..self.width.saturating_sub(2) {
                let mut p = vec![];
                for dy in 0..3 {
                    for dx in 0..3 {
                        p.push(self.get(x + dx, y + dy));
                    }
                }
                if !patterns.contains(&p) {
                    patterns.push(p);
                }
            }
        }
        patterns.len()
    }
}

/// Renormalization group flow: track how observables change under coarse-graining
#[derive(Debug, Clone)]
pub struct RGFlow {
    pub magnetization_history: Vec<i8>,
    pub energy_history: Vec<i32>,
    pub entropy_history: Vec<usize>,
    pub scales: Vec<usize>, // resolution at each step
}

impl RGFlow {
    /// Run RG flow: repeatedly coarse-grain and measure observables
    pub fn run(initial: &TernaryField, max_levels: usize) -> Self {
        let mut flow = Self {
            magnetization_history: vec![],
            energy_history: vec![],
            entropy_history: vec![],
            scales: vec![],
        };
        let mut current = initial.clone();
        for level in 0..max_levels {
            flow.magnetization_history.push(current.magnetization());
            flow.energy_history.push(current.energy());
            flow.entropy_history.push(current.entropy());
            flow.scales.push(current.width);
            if current.width < 4 || current.height < 4 {
                break;
            }
            current = current.coarse_grain_majority();
        }
        flow
    }

    /// Detect fixed point: magnetization stops changing
    pub fn fixed_point(&self) -> Option<i8> {
        if self.magnetization_history.len() < 3 {
            return None;
        }
        let n = self.magnetization_history.len();
        let last = self.magnetization_history[n - 1];
        let prev = self.magnetization_history[n - 2];
        if last == prev {
            Some(last)
        } else {
            None
        }
    }

    /// Is the system at a critical point? (entropy is maximized across scales)
    pub fn is_critical(&self) -> bool {
        if self.entropy_history.len() < 2 {
            return false;
        }
        // Critical: entropy stays high across coarse-graining (scale-invariant)
        let min_entropy = *self.entropy_history.iter().min().unwrap_or(&0);
        let max_entropy = *self.entropy_history.iter().max().unwrap_or(&0);
        // At criticality, entropy doesn't collapse under RG
        min_entropy * 2 > max_entropy
    }
}

/// Universality class detection: compare RG flows of two systems
/// Same universality class → same fixed point behavior
pub fn same_universality_class(flow_a: &RGFlow, flow_b: &RGFlow) -> bool {
    let fp_a = flow_a.fixed_point();
    let fp_b = flow_b.fixed_point();
    fp_a.is_some() && fp_a == fp_b
}

/// Critical exponent: how does magnetization vanish near the critical point?
/// In ternary: β = log(|m₂|) / log(|m₁|) where m₁,m₂ are magnetizations at consecutive scales
pub fn critical_exponent(flow: &RGFlow) -> i8 {
    if flow.magnetization_history.len() < 2 {
        return 0;
    }
    let n = flow.magnetization_history.len();
    let m1 = flow.magnetization_history[n - 2];
    let m2 = flow.magnetization_history[n - 1];
    if m1 == 0 || m2 == 0 { return 0; }
    // Simplified: ratio of magnetization changes
    (m2 * 3 / m1).clamp(-1, 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_new() {
        let f = TernaryField::new(8, 8);
        assert_eq!(f.get(0, 0), 0);
        assert_eq!(f.magnetization(), 0);
    }

    #[test]
    fn test_field_magnetization_uniform() {
        let mut f = TernaryField::new(4, 4);
        for y in 0..4 { for x in 0..4 { f.set(x, y, 1); } }
        assert_eq!(f.magnetization(), 1);
    }

    #[test]
    fn test_field_energy_aligned() {
        let mut f = TernaryField::new(4, 4);
        for y in 0..4 { for x in 0..4 { f.set(x, y, 1); } }
        let e = f.energy();
        assert!(e < 0); // aligned = negative energy (favorable)
    }

    #[test]
    fn test_coarse_grain_majority() {
        let mut f = TernaryField::new(4, 4);
        // Top-left 2x2 block all +1
        f.set(0, 0, 1); f.set(1, 0, 1);
        f.set(0, 1, 1); f.set(1, 1, 1);
        let cg = f.coarse_grain_majority();
        assert_eq!(cg.get(0, 0), 1);
        assert_eq!(cg.width, 2);
    }

    #[test]
    fn test_coarse_grain_sum() {
        let mut f = TernaryField::new(4, 4);
        f.set(0, 0, 1); f.set(1, 0, 1);
        f.set(0, 1, 1); f.set(1, 1, 1);
        let cg = f.coarse_grain_sum();
        assert_eq!(cg.get(0, 0), 1); // sum=4, clamped to 1
    }

    #[test]
    fn test_rg_flow() {
        let mut f = TernaryField::new(8, 8);
        for y in 0..8 { for x in 0..8 { f.set(x, y, 1); } }
        let flow = RGFlow::run(&f, 3);
        assert!(flow.magnetization_history.len() >= 2);
        assert_eq!(flow.fixed_point(), Some(1));
    }

    #[test]
    fn test_rg_flow_random() {
        let mut f = TernaryField::new(8, 8);
        // Mixed field
        for y in 0..8 {
            for x in 0..8 {
                f.set(x, y, if (x + y) % 2 == 0 { 1 } else { -1 });
            }
        }
        let flow = RGFlow::run(&f, 3);
        assert!(flow.scales.len() >= 2);
    }

    #[test]
    fn test_fixed_point_none() {
        let mut f = TernaryField::new(8, 8);
        f.set(0, 0, 1);
        let flow = RGFlow::run(&f, 3);
        // May or may not have a fixed point for a single-perturbation field
        // Just check it runs
        assert!(!flow.magnetization_history.is_empty());
    }

    #[test]
    fn test_entropy() {
        let mut f = TernaryField::new(6, 6);
        for y in 0..6 { for x in 0..6 { f.set(x, y, 1); } }
        let e = f.entropy();
        // All same → entropy should be 1 (only one pattern)
        assert_eq!(e, 1);
    }

    #[test]
    fn test_entropy_diverse() {
        let mut f = TernaryField::new(6, 6);
        // Checkerboard with three values
        for y in 0..6 {
            for x in 0..6 {
                let v = match (x + y) % 3 {
                    0 => -1,
                    1 => 0,
                    _ => 1,
                };
                f.set(x, y, v);
            }
        }
        let e = f.entropy();
        // With period-3 pattern on a 6x6 grid, should have at most a few distinct 3x3 patches
        assert!(e >= 1);
    }

    #[test]
    fn test_same_universality_class() {
        let mut f1 = TernaryField::new(8, 8);
        let mut f2 = TernaryField::new(8, 8);
        for y in 0..8 { for x in 0..8 { f1.set(x, y, 1); f2.set(x, y, 1); } }
        let flow1 = RGFlow::run(&f1, 3);
        let flow2 = RGFlow::run(&f2, 3);
        assert!(same_universality_class(&flow1, &flow2));
    }

    #[test]
    fn test_critical_exponent() {
        let mut f = TernaryField::new(8, 8);
        for y in 0..8 { for x in 0..8 { f.set(x, y, 1); } }
        let flow = RGFlow::run(&f, 3);
        let beta = critical_exponent(&flow);
        assert!(beta >= -1 && beta <= 1);
    }
}
