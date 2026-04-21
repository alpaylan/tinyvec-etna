//! ETNA framework-neutral property functions for the `tinyvec` crate.
//!
//! Each `property_<name>` is a pure function over concrete, owned inputs
//! returning `PropertyResult`. Framework adapters in `src/bin/etna.rs` and
//! witness tests in `tests/etna_witnesses.rs` call these directly.

#![allow(missing_docs)]

use crate::ArrayVec;

use std::format;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::string::{String, ToString};
use std::vec::Vec;

#[derive(Debug)]
pub enum PropertyResult {
    Pass,
    Fail(String),
    Discard,
}

// ---------------------------------------------------------------------------
// debug_alternate_empty_a711c72_1
//
// ArrayVec's `Debug` impl (alternate mode, `"{:#?}"`) must produce the same
// output as formatting the underlying slice. The pre-a711c72 impl emitted
// `"[\n    ,\n]"` on an empty vec because the leading-newline / trailing-
// comma-newline blocks were not guarded by `!self.is_empty()`. Later fixes
// (a6db082) replace the manual impl with a delegation to `[T]::fmt`, which
// inherits the correct behavior.
//
// Invariant: `format!("{:#?}", av) == format!("{:#?}", av.as_slice())`.
// ---------------------------------------------------------------------------

pub fn property_arrayvec_debug_matches_slice(items: Vec<i32>) -> PropertyResult {
    const CAP: usize = 8;

    if items.len() > CAP {
        return PropertyResult::Discard;
    }

    let mut av: ArrayVec<[i32; CAP]> = ArrayVec::new();
    for x in items.iter().copied() {
        av.push(x);
    }

    let slice_alt = format!("{:#?}", av.as_slice());
    let vec_alt = format!("{:#?}", av);
    if vec_alt != slice_alt {
        return PropertyResult::Fail(format!(
            "alternate Debug mismatch: av={:?} slice={:?}",
            vec_alt, slice_alt
        ));
    }

    let slice_plain = format!("{:?}", av.as_slice());
    let vec_plain = format!("{:?}", av);
    if vec_plain != slice_plain {
        return PropertyResult::Fail(format!(
            "Debug mismatch: av={:?} slice={:?}",
            vec_plain, slice_plain
        ));
    }

    PropertyResult::Pass
}

// ---------------------------------------------------------------------------
// remove_past_end_silent_fd3c92c_1
//
// `ArrayVec::remove(index)` must panic when `index >= self.len()` — the
// standard `Vec::remove` contract. The pre-fd3c92c implementation silently
// returned `A::Item::default()` for `index == self.len()` because it relied
// on `self.deref_mut()[index..]` producing an empty slice and then only
// decremented `self.len` without panicking. The fix reads `targets[0]`,
// which panics on the empty slice.
//
// Invariant: `remove(len)` must panic on a non-empty ArrayVec.
// ---------------------------------------------------------------------------

pub fn property_remove_past_end_panics(items: Vec<i32>) -> PropertyResult {
    const CAP: usize = 8;

    if items.is_empty() || items.len() > CAP {
        return PropertyResult::Discard;
    }

    let mut av: ArrayVec<[i32; CAP]> = ArrayVec::new();
    for x in items.iter().copied() {
        av.push(x);
    }
    let len = av.len();

    let result = catch_unwind(AssertUnwindSafe(move || av.remove(len)));
    match result {
        Err(_) => PropertyResult::Pass,
        Ok(v) => PropertyResult::Fail(format!(
            "ArrayVec::remove({}) on len={} did not panic; returned {}",
            len, len, v
        )),
    }
}

// ---------------------------------------------------------------------------
// swap_remove_last_71ad62a_1
//
// `ArrayVec::swap_remove(len-1)` must return the last element without
// panicking. The pre-71ad62a implementation did
//   let i = self.pop().unwrap();
//   replace(&mut self[index], i)
// which, when `index == len - 1`, pops (leaving len-1 elements) and then
// tries to index `[len-1]` which is out of bounds, panicking. The fix
// special-cases `index == len - 1` to just `pop().unwrap()`.
//
// Invariant: for a non-empty vec, `swap_remove(len-1)` returns the tail
// element and shrinks the vec by one, without panicking.
// ---------------------------------------------------------------------------

pub fn property_swap_remove_last_returns_tail(items: Vec<i32>) -> PropertyResult {
    const CAP: usize = 8;

    if items.is_empty() || items.len() > CAP {
        return PropertyResult::Discard;
    }

    let mut av: ArrayVec<[i32; CAP]> = ArrayVec::new();
    for x in items.iter().copied() {
        av.push(x);
    }
    let expected_tail = *items.last().unwrap();
    let expected_prefix: Vec<i32> = items[..items.len() - 1].to_vec();
    let last_index = av.len() - 1;

    let result = catch_unwind(AssertUnwindSafe(move || {
        let tail = av.swap_remove(last_index);
        let rest: Vec<i32> = av.iter().copied().collect();
        (tail, rest)
    }));

    match result {
        Err(_) => PropertyResult::Fail(format!(
            "swap_remove({}) on len={} panicked unexpectedly",
            last_index,
            last_index + 1
        )),
        Ok((tail, rest)) => {
            if tail != expected_tail {
                return PropertyResult::Fail(format!(
                    "swap_remove returned {} expected {}",
                    tail, expected_tail
                ));
            }
            if rest != expected_prefix {
                return PropertyResult::Fail(format!(
                    "after swap_remove tail, rest={:?} expected {:?}",
                    rest, expected_prefix
                ));
            }
            PropertyResult::Pass
        }
    }
}

// ---------------------------------------------------------------------------
// drain_end_off_by_one_9117614_1
//
// `ArrayVec::drain(a..b)` must yield exactly the elements in `items[a..b]`
// — the same semantics as `Vec::drain`. Pre-9117614 the end-bound
// interpretation was off by one:
//   Bound::Included(x) => *x      (wrong: should be x + 1)
//   Bound::Excluded(x) => x - 1   (wrong: should be *x)
// Which meant a half-open range `a..b` drained `a..b-1` instead of `a..b`,
// and an inclusive `a..=b` drained `a..b` instead of `a..=b`.
//
// Invariant: `av.drain(a..b).collect::<Vec<_>>() == items[a..b].to_vec()`
// and after the drain `av` holds `items[..a].to_vec() ++ items[b..].to_vec()`.
// We exercise both `Range` and `RangeInclusive` to catch either mis-mapping.
// ---------------------------------------------------------------------------

pub fn property_drain_matches_slice_range(
    items: Vec<i32>,
    a: usize,
    b: usize,
) -> PropertyResult {
    const CAP: usize = 8;

    if items.is_empty() || items.len() > CAP {
        return PropertyResult::Discard;
    }
    // Keep start <= end <= len so the range is well-formed; the property
    // tests the end-bound mapping, not the range-assertion errors.
    let end = b % (items.len() + 1);
    let start = if end == 0 { 0 } else { a % (end + 1) };
    if start > end {
        return PropertyResult::Discard;
    }

    // Half-open: drain(start..end)
    {
        let items = items.clone();
        let expected_drained: Vec<i32> = items[start..end].to_vec();
        let mut expected_rest: Vec<i32> = items[..start].to_vec();
        expected_rest.extend_from_slice(&items[end..]);

        let mut av: ArrayVec<[i32; CAP]> = ArrayVec::new();
        for x in items.iter().copied() {
            av.push(x);
        }
        let drained: Vec<i32> = av.drain(start..end).collect();
        let rest: Vec<i32> = av.iter().copied().collect();

        if drained != expected_drained {
            return PropertyResult::Fail(format!(
                "drain({}..{}) yielded {:?} expected {:?}",
                start, end, drained, expected_drained
            ));
        }
        if rest != expected_rest {
            return PropertyResult::Fail(format!(
                "after drain({}..{}), rest={:?} expected {:?}",
                start, end, rest, expected_rest
            ));
        }
    }

    // Inclusive: drain(start..=end-1) when end>0, matches items[start..end].
    if end > 0 && start <= end - 1 {
        let end_inc = end - 1;
        let items = items.clone();
        let expected_drained: Vec<i32> = items[start..=end_inc].to_vec();
        let mut expected_rest: Vec<i32> = items[..start].to_vec();
        expected_rest.extend_from_slice(&items[end_inc + 1..]);

        let mut av: ArrayVec<[i32; CAP]> = ArrayVec::new();
        for x in items.iter().copied() {
            av.push(x);
        }
        let drained: Vec<i32> = av.drain(start..=end_inc).collect();
        let rest: Vec<i32> = av.iter().copied().collect();

        if drained != expected_drained {
            return PropertyResult::Fail(format!(
                "drain({}..={}) yielded {:?} expected {:?}",
                start, end_inc, drained, expected_drained
            ));
        }
        if rest != expected_rest {
            return PropertyResult::Fail(format!(
                "after drain({}..={}), rest={:?} expected {:?}",
                start, end_inc, rest, expected_rest
            ));
        }
    }

    PropertyResult::Pass
}
