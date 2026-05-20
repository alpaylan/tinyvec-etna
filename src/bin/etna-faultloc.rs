// Crabcheck fault-localization runner for tinyvec.
use std::fmt;

use crabcheck::profiling::quickcheck;
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
            1 => {
                let bit = rng.random_range(0u32..(usize::BITS));
                out.a ^= 1usize << bit;
            },
            2 => {
                let bit = rng.random_range(0u32..(usize::BITS));
                out.b ^= 1usize << bit;
            },
            _ => {
                if rng.random_bool(0.5) && out.items.len() < 12 {
                    out.items.push(rng.random::<i32>());
                } else if !out.items.is_empty() {
                    out.items.pop();
                }
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


fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() < 3 { return; }
    let tool = args[1].as_str();
    let property = args[2].as_str();
    let result = match (tool, property) {
        ("crabcheck", "ArrayvecDebugMatchesSlice") => {
            quickcheck(|i: ItemsInput| {
                to_opt(property_arrayvec_debug_matches_slice(i.items))
            })
        },
        ("crabcheck", "RemovePastEndPanics") => {
            quickcheck(|i: ItemsInput| {
                to_opt(property_remove_past_end_panics(i.items))
            })
        },
        ("crabcheck", "SwapRemoveLastReturnsTail") => {
            quickcheck(|i: ItemsInput| {
                to_opt(property_swap_remove_last_returns_tail(i.items))
            })
        },
        ("crabcheck", "DrainMatchesSliceRange") => {
            quickcheck(|i: DrainInput| {
                to_opt(property_drain_matches_slice_range(i.items, i.a, i.b))
            })
        },
        _ => panic!("Unknown: {tool} {property}"),
    };
    println!("Result: {:?}", result);
}
