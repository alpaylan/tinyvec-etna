//! Fault-localization integration tests for tinyvec.

#![cfg(feature = "etna")]

use std::fmt;

use crabcheck::quickcheck::{Arbitrary, Mutate};
use rand_etna::Rng;
use tinyvec::etna::{
    property_arrayvec_debug_matches_slice, property_drain_matches_slice_range,
    property_remove_past_end_panics, property_swap_remove_last_returns_tail, PropertyResult,
};

#[derive(Clone)]
struct ItemsInput { items: Vec<i32> }
impl fmt::Debug for ItemsInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{:?}", self.items) }
}

#[derive(Clone)]
struct DrainInput { items: Vec<i32>, a: usize, b: usize }
impl fmt::Debug for DrainInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} {} {}", self.items, self.a, self.b)
    }
}

impl<R: Rng> Arbitrary<R> for ItemsInput {
    fn generate(rng: &mut R, _n: usize) -> Self {
        let len = (rng.random::<u8>() % 9) as usize;
        ItemsInput { items: (0..len).map(|_| rng.random::<i32>()).collect() }
    }
}

impl<R: Rng> Arbitrary<R> for DrainInput {
    fn generate(rng: &mut R, _n: usize) -> Self {
        let len = (rng.random::<u8>() % 9) as usize;
        DrainInput {
            items: (0..len).map(|_| rng.random::<i32>()).collect(),
            a: rng.random::<u8>() as usize,
            b: rng.random::<u8>() as usize,
        }
    }
}

impl<R: Rng> Mutate<R> for ItemsInput {
    fn mutate(&self, rng: &mut R, _n: usize) -> Self {
        let mut out = self.clone();
        match rng.random_range(0u8..3) {
            0 if !out.items.is_empty() => {
                let i = rng.random_range(0..out.items.len());
                let b = rng.random_range(0u32..32);
                out.items[i] ^= 1i32 << b;
            },
            1 if out.items.len() < 12 => out.items.push(rng.random::<i32>()),
            _ if !out.items.is_empty() => { out.items.pop(); },
            _ => {},
        }
        out
    }
}

impl<R: Rng> Mutate<R> for DrainInput {
    fn mutate(&self, rng: &mut R, _n: usize) -> Self {
        let mut out = self.clone();
        match rng.random_range(0u8..4) {
            0 if !out.items.is_empty() => {
                let i = rng.random_range(0..out.items.len());
                let b = rng.random_range(0u32..32);
                out.items[i] ^= 1i32 << b;
            },
            1 => { let bit = rng.random_range(0u32..(usize::BITS)); out.a ^= 1usize << bit; },
            2 => { let bit = rng.random_range(0u32..(usize::BITS)); out.b ^= 1usize << bit; },
            _ => {
                if rng.random_bool(0.5) && out.items.len() < 12 {
                    out.items.push(rng.random::<i32>());
                } else if !out.items.is_empty() { out.items.pop(); }
            },
        }
        out
    }
}

fn to_opt(r: PropertyResult) -> Option<bool> {
    match r {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn property_arrayvec_debug_matches_slice_test(i: ItemsInput) -> Option<bool> {
    to_opt(property_arrayvec_debug_matches_slice(i.items))
}
fn property_remove_past_end_panics_test(i: ItemsInput) -> Option<bool> {
    to_opt(property_remove_past_end_panics(i.items))
}
fn property_swap_remove_last_returns_tail_test(i: ItemsInput) -> Option<bool> {
    to_opt(property_swap_remove_last_returns_tail(i.items))
}
fn property_drain_matches_slice_range_test(i: DrainInput) -> Option<bool> {
    to_opt(property_drain_matches_slice_range(i.items, i.a, i.b))
}

fn emit_locate_json(r: &crabcheck::profiling::LocateResult) {
    use crabcheck::quickcheck::ResultStatus;
    let status = match &r.run.status {
        ResultStatus::Failed { .. } => "Failed",
        ResultStatus::Finished => "Finished",
        ResultStatus::GaveUp => "GaveUp",
        ResultStatus::TimedOut => "TimedOut",
        ResultStatus::Aborted { .. } => "Aborted",
    };
    let top = if let Some(s) = r.top() {
        serde_json::json!({
            "rank": s.rank, "file": s.region.file, "function": s.region.function,
            "start_line": s.region.start_line, "end_line": s.region.end_line,
            "ochiai": s.region.suspiciousness.ochiai, "delta": s.region.delta,
            "panic_overlap": s.panic_overlap,
            "confidence": format!("{}", s.confidence),
            "confidence_rule": s.confidence_rule,
        })
    } else { serde_json::Value::Null };
    let top_5: Vec<_> = r.suspects.iter().take(5).map(|s| serde_json::json!({
        "rank": s.rank, "file": s.region.file, "function": s.region.function,
        "start_line": s.region.start_line, "end_line": s.region.end_line,
        "confidence": format!("{}", s.confidence),
        "confidence_rule": s.confidence_rule,
        "panic_overlap": s.panic_overlap,
    })).collect();
    let diags: Vec<_> = r.diagnostics.iter().map(|d| d.tag()).collect();
    let out = serde_json::json!({
        "status": status, "passed": r.run.passed, "discarded": r.run.discarded,
        "n_panics": r.n_panics, "n_suspects": r.suspects.len(),
        "top": top, "top_5": top_5, "diagnostics": diags,
    });
    println!("@@LOCATE@@ {}", out);
}

#[test]
fn locate_arrayvec_debug_matches_slice() {
    let report = crabcheck::quickcheck_with_locate!(property_arrayvec_debug_matches_slice_test, "tinyvec");
    eprintln!("{report}");
    emit_locate_json(&report);
}

#[test]
fn locate_remove_past_end_panics() {
    let report = crabcheck::quickcheck_with_locate!(property_remove_past_end_panics_test, "tinyvec");
    eprintln!("{report}");
    emit_locate_json(&report);
}

#[test]
fn locate_swap_remove_last_returns_tail() {
    let report = crabcheck::quickcheck_with_locate!(property_swap_remove_last_returns_tail_test, "tinyvec");
    eprintln!("{report}");
    emit_locate_json(&report);
}

#[test]
fn locate_drain_matches_slice_range() {
    let report = crabcheck::quickcheck_with_locate!(property_drain_matches_slice_range_test, "tinyvec");
    eprintln!("{report}");
    emit_locate_json(&report);
}
