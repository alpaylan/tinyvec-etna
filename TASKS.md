# tinyvec — ETNA Tasks

Total tasks: 16

## Task Index

| Task | Variant | Framework | Property | Witness |
|------|---------|-----------|----------|---------|
| 001 | `debug_alternate_empty_a711c72_1` | proptest | `ArrayvecDebugMatchesSlice` | `witness_arrayvec_debug_matches_slice_case_empty` |
| 002 | `debug_alternate_empty_a711c72_1` | quickcheck | `ArrayvecDebugMatchesSlice` | `witness_arrayvec_debug_matches_slice_case_empty` |
| 003 | `debug_alternate_empty_a711c72_1` | crabcheck | `ArrayvecDebugMatchesSlice` | `witness_arrayvec_debug_matches_slice_case_empty` |
| 004 | `debug_alternate_empty_a711c72_1` | hegel | `ArrayvecDebugMatchesSlice` | `witness_arrayvec_debug_matches_slice_case_empty` |
| 005 | `drain_end_off_by_one_9117614_1` | proptest | `DrainMatchesSliceRange` | `witness_drain_matches_slice_range_case_exclusive_middle` |
| 006 | `drain_end_off_by_one_9117614_1` | quickcheck | `DrainMatchesSliceRange` | `witness_drain_matches_slice_range_case_exclusive_middle` |
| 007 | `drain_end_off_by_one_9117614_1` | crabcheck | `DrainMatchesSliceRange` | `witness_drain_matches_slice_range_case_exclusive_middle` |
| 008 | `drain_end_off_by_one_9117614_1` | hegel | `DrainMatchesSliceRange` | `witness_drain_matches_slice_range_case_exclusive_middle` |
| 009 | `remove_past_end_silent_fd3c92c_1` | proptest | `RemovePastEndPanics` | `witness_remove_past_end_panics_case_single_element` |
| 010 | `remove_past_end_silent_fd3c92c_1` | quickcheck | `RemovePastEndPanics` | `witness_remove_past_end_panics_case_single_element` |
| 011 | `remove_past_end_silent_fd3c92c_1` | crabcheck | `RemovePastEndPanics` | `witness_remove_past_end_panics_case_single_element` |
| 012 | `remove_past_end_silent_fd3c92c_1` | hegel | `RemovePastEndPanics` | `witness_remove_past_end_panics_case_single_element` |
| 013 | `swap_remove_last_71ad62a_1` | proptest | `SwapRemoveLastReturnsTail` | `witness_swap_remove_last_returns_tail_case_single` |
| 014 | `swap_remove_last_71ad62a_1` | quickcheck | `SwapRemoveLastReturnsTail` | `witness_swap_remove_last_returns_tail_case_single` |
| 015 | `swap_remove_last_71ad62a_1` | crabcheck | `SwapRemoveLastReturnsTail` | `witness_swap_remove_last_returns_tail_case_single` |
| 016 | `swap_remove_last_71ad62a_1` | hegel | `SwapRemoveLastReturnsTail` | `witness_swap_remove_last_returns_tail_case_single` |

## Witness Catalog

- `witness_arrayvec_debug_matches_slice_case_empty` — base passes, variant fails
- `witness_arrayvec_debug_matches_slice_case_three_elements` — base passes, variant fails
- `witness_drain_matches_slice_range_case_exclusive_middle` — base passes, variant fails
- `witness_drain_matches_slice_range_case_full_range` — base passes, variant fails
- `witness_remove_past_end_panics_case_single_element` — base passes, variant fails
- `witness_remove_past_end_panics_case_three_elements` — base passes, variant fails
- `witness_swap_remove_last_returns_tail_case_single` — base passes, variant fails
- `witness_swap_remove_last_returns_tail_case_four_elements` — base passes, variant fails
