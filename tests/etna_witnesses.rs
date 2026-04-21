//! Deterministic witness tests for tinyvec ETNA variants.
//!
//! Each `witness_<name>_case_<tag>` passes on base HEAD and fails under the
//! corresponding `etna/<variant>` branch (or `M_<variant>=active`). Witnesses
//! call `property_<name>` directly with frozen inputs — no generators.

#![cfg(feature = "etna")]

use tinyvec::etna::{
    property_arrayvec_debug_matches_slice, property_drain_matches_slice_range,
    property_remove_past_end_panics, property_swap_remove_last_returns_tail, PropertyResult,
};

fn expect_pass(r: PropertyResult, what: &str) {
    match r {
        PropertyResult::Pass => {}
        PropertyResult::Fail(m) => panic!("{}: property failed: {}", what, m),
        PropertyResult::Discard => panic!("{}: unexpected discard", what),
    }
}

// ---- debug_alternate_empty_a711c72_1 -------------------------------------

#[test]
fn witness_arrayvec_debug_matches_slice_case_empty() {
    expect_pass(
        property_arrayvec_debug_matches_slice(vec![]),
        "Debug on empty ArrayVec matches empty slice",
    );
}

#[test]
fn witness_arrayvec_debug_matches_slice_case_three_elements() {
    expect_pass(
        property_arrayvec_debug_matches_slice(vec![1, 2, 3]),
        "Debug on three-element ArrayVec matches slice",
    );
}

// ---- remove_past_end_silent_fd3c92c_1 ------------------------------------

#[test]
fn witness_remove_past_end_panics_case_single_element() {
    expect_pass(
        property_remove_past_end_panics(vec![42]),
        "remove(1) on a single-element vec panics",
    );
}

#[test]
fn witness_remove_past_end_panics_case_three_elements() {
    expect_pass(
        property_remove_past_end_panics(vec![1, 2, 3]),
        "remove(3) on a three-element vec panics",
    );
}

// ---- swap_remove_last_71ad62a_1 ------------------------------------------

#[test]
fn witness_swap_remove_last_returns_tail_case_single() {
    expect_pass(
        property_swap_remove_last_returns_tail(vec![99]),
        "swap_remove(0) on a single-element vec returns the tail",
    );
}

#[test]
fn witness_swap_remove_last_returns_tail_case_four_elements() {
    expect_pass(
        property_swap_remove_last_returns_tail(vec![1, 2, 3, 4]),
        "swap_remove(3) on a four-element vec returns the tail",
    );
}

// ---- drain_end_off_by_one_9117614_1 --------------------------------------

#[test]
fn witness_drain_matches_slice_range_case_exclusive_middle() {
    // start=1, end=3 on items=[1,2,3,4,5] drains [2,3].
    expect_pass(
        property_drain_matches_slice_range(vec![1, 2, 3, 4, 5], 1, 3),
        "drain(1..3) on [1..=5] drains [2,3]",
    );
}

#[test]
fn witness_drain_matches_slice_range_case_full_range() {
    // start=0, end=len drains everything.
    expect_pass(
        property_drain_matches_slice_range(vec![10, 20, 30], 0, 3),
        "drain(0..3) on [10,20,30] drains all three",
    );
}
