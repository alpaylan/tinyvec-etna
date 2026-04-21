# tinyvec — ETNA Tasks

Total tasks: 16

ETNA tasks are **mutation/property/witness triplets**. Each row below is one runnable task.

## Task Index

| Task | Variant | Framework | Property | Witness | Command |
|------|---------|-----------|----------|---------|---------|
| 001  | `debug_alternate_empty_a711c72_1`  | proptest   | `property_arrayvec_debug_matches_slice`    | `witness_arrayvec_debug_matches_slice_case_empty`                | `cargo run --release --features etna --bin etna -- proptest ArrayvecDebugMatchesSlice` |
| 002  | `debug_alternate_empty_a711c72_1`  | quickcheck | `property_arrayvec_debug_matches_slice`    | `witness_arrayvec_debug_matches_slice_case_empty`                | `cargo run --release --features etna --bin etna -- quickcheck ArrayvecDebugMatchesSlice` |
| 003  | `debug_alternate_empty_a711c72_1`  | crabcheck  | `property_arrayvec_debug_matches_slice`    | `witness_arrayvec_debug_matches_slice_case_empty`                | `cargo run --release --features etna --bin etna -- crabcheck ArrayvecDebugMatchesSlice` |
| 004  | `debug_alternate_empty_a711c72_1`  | hegel      | `property_arrayvec_debug_matches_slice`    | `witness_arrayvec_debug_matches_slice_case_empty`                | `cargo run --release --features etna --bin etna -- hegel ArrayvecDebugMatchesSlice` |
| 005  | `remove_past_end_silent_fd3c92c_1` | proptest   | `property_remove_past_end_panics`          | `witness_remove_past_end_panics_case_three_elements`             | `cargo run --release --features etna --bin etna -- proptest RemovePastEndPanics` |
| 006  | `remove_past_end_silent_fd3c92c_1` | quickcheck | `property_remove_past_end_panics`          | `witness_remove_past_end_panics_case_three_elements`             | `cargo run --release --features etna --bin etna -- quickcheck RemovePastEndPanics` |
| 007  | `remove_past_end_silent_fd3c92c_1` | crabcheck  | `property_remove_past_end_panics`          | `witness_remove_past_end_panics_case_three_elements`             | `cargo run --release --features etna --bin etna -- crabcheck RemovePastEndPanics` |
| 008  | `remove_past_end_silent_fd3c92c_1` | hegel      | `property_remove_past_end_panics`          | `witness_remove_past_end_panics_case_three_elements`             | `cargo run --release --features etna --bin etna -- hegel RemovePastEndPanics` |
| 009  | `swap_remove_last_71ad62a_1`       | proptest   | `property_swap_remove_last_returns_tail`   | `witness_swap_remove_last_returns_tail_case_four_elements`       | `cargo run --release --features etna --bin etna -- proptest SwapRemoveLastReturnsTail` |
| 010  | `swap_remove_last_71ad62a_1`       | quickcheck | `property_swap_remove_last_returns_tail`   | `witness_swap_remove_last_returns_tail_case_four_elements`       | `cargo run --release --features etna --bin etna -- quickcheck SwapRemoveLastReturnsTail` |
| 011  | `swap_remove_last_71ad62a_1`       | crabcheck  | `property_swap_remove_last_returns_tail`   | `witness_swap_remove_last_returns_tail_case_four_elements`       | `cargo run --release --features etna --bin etna -- crabcheck SwapRemoveLastReturnsTail` |
| 012  | `swap_remove_last_71ad62a_1`       | hegel      | `property_swap_remove_last_returns_tail`   | `witness_swap_remove_last_returns_tail_case_four_elements`       | `cargo run --release --features etna --bin etna -- hegel SwapRemoveLastReturnsTail` |
| 013  | `drain_end_off_by_one_9117614_1`   | proptest   | `property_drain_matches_slice_range`       | `witness_drain_matches_slice_range_case_exclusive_middle`        | `cargo run --release --features etna --bin etna -- proptest DrainMatchesSliceRange` |
| 014  | `drain_end_off_by_one_9117614_1`   | quickcheck | `property_drain_matches_slice_range`       | `witness_drain_matches_slice_range_case_exclusive_middle`        | `cargo run --release --features etna --bin etna -- quickcheck DrainMatchesSliceRange` |
| 015  | `drain_end_off_by_one_9117614_1`   | crabcheck  | `property_drain_matches_slice_range`       | `witness_drain_matches_slice_range_case_exclusive_middle`        | `cargo run --release --features etna --bin etna -- crabcheck DrainMatchesSliceRange` |
| 016  | `drain_end_off_by_one_9117614_1`   | hegel      | `property_drain_matches_slice_range`       | `witness_drain_matches_slice_range_case_exclusive_middle`        | `cargo run --release --features etna --bin etna -- hegel DrainMatchesSliceRange` |

## Witness catalog

Each witness is a deterministic concrete test. Base build: passes. Variant-active build: fails.

- `witness_arrayvec_debug_matches_slice_case_empty` — `format!("{:#?}", av)` on an empty `ArrayVec<[i32; 8]>`. Base emits `"[]"` (inheriting `[T]::fmt`); the variant emits `"[\n    ,\n]"` thanks to unconditional leading newline + trailing comma/newline.
- `witness_arrayvec_debug_matches_slice_case_three_elements` — same property on `ArrayVec<[i32; 8]>` pre-loaded with `[1, 2, 3]`. Base produces `[T]`'s newline-per-item layout; the variant produces a single-line layout with a trailing comma — layouts differ, property fails.
- `witness_remove_past_end_panics_case_single_element` — `remove(1)` on an `ArrayVec<[i32; 8]>` pre-loaded with `[42]`. Base panics (index-out-of-bounds on `targets[0]`); the variant silently returns the default `i32` (`0`) and decrements `self.len`.
- `witness_remove_past_end_panics_case_three_elements` — `remove(3)` on `ArrayVec<[i32; 8]>` pre-loaded with `[1, 2, 3]`. Base panics; the variant silently returns `0`.
- `witness_swap_remove_last_returns_tail_case_single` — `swap_remove(0)` on a single-element `ArrayVec<[i32; 8]>` `[99]`. Base returns `99` and leaves an empty vec; the variant panics because the `self.pop()` shrinks to zero and `self[0]` is then out of range.
- `witness_swap_remove_last_returns_tail_case_four_elements` — `swap_remove(3)` on `ArrayVec<[i32; 8]>` `[1,2,3,4]`. Base returns `4` with `[1,2,3]` remaining; the variant panics.
- `witness_drain_matches_slice_range_case_exclusive_middle` — `drain(1..3)` on `ArrayVec<[i32; 8]>` `[1,2,3,4,5]`. Base drains `[2,3]`; the variant drains only `[2]` (end-bound off by one on the excluded bound).
- `witness_drain_matches_slice_range_case_full_range` — `drain(0..3)` on `ArrayVec<[i32; 8]>` `[10,20,30]`. Base drains all three and leaves an empty vec; the variant drains only `[10,20]` and leaves `[30]`.
