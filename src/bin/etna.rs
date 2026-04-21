// ETNA workload runner for tinyvec.
//
// Usage: cargo run --release --bin etna -- <tool> <property>
//   tool:     etna | proptest | quickcheck | crabcheck | hegel
//   property: ArrayvecDebugMatchesSlice
//           | RemovePastEndPanics
//           | SwapRemoveLastReturnsTail
//           | DrainMatchesSliceRange
//           | All
//
// Each invocation emits a single JSON line on stdout and exits 0 (usage
// errors exit 2). Adapters drive their own framework crate directly —
// no subprocess dispatch.

use tinyvec::etna::{
    property_arrayvec_debug_matches_slice, property_drain_matches_slice_range,
    property_remove_past_end_panics, property_swap_remove_last_returns_tail, PropertyResult,
};

use crabcheck::quickcheck as crabcheck_qc;
use crabcheck::quickcheck::Arbitrary as CcArbitrary;
use hegel::{generators as hgen, HealthCheck, Hegel, Settings as HegelSettings, TestCase};
use proptest::prelude::*;
use proptest::test_runner::{Config as ProptestConfig, TestCaseError, TestError};
use quickcheck_etna::{Arbitrary as QcArbitrary, Gen, QuickCheck, ResultStatus, TestResult};
use rand_etna::Rng;

use std::fmt;
use std::panic::AssertUnwindSafe;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Default, Clone, Copy)]
struct Metrics {
    inputs: u64,
    elapsed_us: u128,
}

impl Metrics {
    fn combine(self, other: Metrics) -> Metrics {
        Metrics {
            inputs: self.inputs + other.inputs,
            elapsed_us: self.elapsed_us + other.elapsed_us,
        }
    }
}

type Outcome = (Result<(), String>, Metrics);

fn to_err(r: PropertyResult) -> Result<(), String> {
    match r {
        PropertyResult::Pass | PropertyResult::Discard => Ok(()),
        PropertyResult::Fail(m) => Err(m),
    }
}

const ALL_PROPERTIES: &[&str] = &[
    "ArrayvecDebugMatchesSlice",
    "RemovePastEndPanics",
    "SwapRemoveLastReturnsTail",
    "DrainMatchesSliceRange",
];

fn cases_budget() -> u64 {
    std::env::var("ETNA_CASES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1_000)
}

fn run_all<F: FnMut(&str) -> Outcome>(mut f: F) -> Outcome {
    let mut total = Metrics::default();
    for p in ALL_PROPERTIES {
        let (r, m) = f(p);
        total = total.combine(m);
        if let Err(e) = r {
            return (Err(e), total);
        }
    }
    (Ok(()), total)
}

// ============================================================================
// Input wrappers
// ============================================================================

#[derive(Clone)]
struct ItemsInput {
    items: Vec<i32>,
}

impl fmt::Debug for ItemsInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.items)
    }
}

impl fmt::Display for ItemsInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Clone)]
struct DrainInput {
    items: Vec<i32>,
    a: usize,
    b: usize,
}

impl fmt::Debug for DrainInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} {} {}", self.items, self.a, self.b)
    }
}

impl fmt::Display for DrainInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

// ============================================================================
// Canonical witness inputs — used by `tool=etna` to replay frozen cases.
// ============================================================================

fn check_arrayvec_debug_matches_slice() -> Result<(), String> {
    to_err(property_arrayvec_debug_matches_slice(vec![]))?;
    to_err(property_arrayvec_debug_matches_slice(vec![1, 2, 3]))?;
    Ok(())
}

fn check_remove_past_end_panics() -> Result<(), String> {
    to_err(property_remove_past_end_panics(vec![42]))?;
    to_err(property_remove_past_end_panics(vec![1, 2, 3]))?;
    Ok(())
}

fn check_swap_remove_last_returns_tail() -> Result<(), String> {
    to_err(property_swap_remove_last_returns_tail(vec![99]))?;
    to_err(property_swap_remove_last_returns_tail(vec![1, 2, 3, 4]))?;
    Ok(())
}

fn check_drain_matches_slice_range() -> Result<(), String> {
    to_err(property_drain_matches_slice_range(
        vec![1, 2, 3, 4, 5],
        1,
        3,
    ))?;
    to_err(property_drain_matches_slice_range(vec![10, 20, 30], 0, 3))?;
    Ok(())
}

fn panic_msg(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else if let Some(s) = payload.downcast_ref::<&str>() {
        s.to_string()
    } else {
        "panic with non-string payload".to_string()
    }
}

fn run_etna_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_etna_property);
    }
    let t0 = Instant::now();
    let result = std::panic::catch_unwind(AssertUnwindSafe(|| match property {
        "ArrayvecDebugMatchesSlice" => check_arrayvec_debug_matches_slice(),
        "RemovePastEndPanics" => check_remove_past_end_panics(),
        "SwapRemoveLastReturnsTail" => check_swap_remove_last_returns_tail(),
        "DrainMatchesSliceRange" => check_drain_matches_slice_range(),
        _ => Err(format!("Unknown property for etna: {}", property)),
    }));
    let elapsed_us = t0.elapsed().as_micros();
    let status = match result {
        Ok(r) => r,
        Err(payload) => Err(panic_msg(payload)),
    };
    (status, Metrics { inputs: 1, elapsed_us })
}

// ============================================================================
// quickcheck Arbitrary
// ============================================================================

impl QcArbitrary for ItemsInput {
    fn arbitrary(g: &mut Gen) -> Self {
        let len = (<u8 as QcArbitrary>::arbitrary(g) % 9) as usize;
        let mut items = Vec::with_capacity(len);
        for _ in 0..len {
            items.push(<i32 as QcArbitrary>::arbitrary(g));
        }
        ItemsInput { items }
    }
}

impl QcArbitrary for DrainInput {
    fn arbitrary(g: &mut Gen) -> Self {
        let len = (<u8 as QcArbitrary>::arbitrary(g) % 9) as usize;
        let mut items = Vec::with_capacity(len);
        for _ in 0..len {
            items.push(<i32 as QcArbitrary>::arbitrary(g));
        }
        let a = <u8 as QcArbitrary>::arbitrary(g) as usize;
        let b = <u8 as QcArbitrary>::arbitrary(g) as usize;
        DrainInput { items, a, b }
    }
}

// ============================================================================
// crabcheck Arbitrary
// ============================================================================

impl<R: Rng> CcArbitrary<R> for ItemsInput {
    fn generate(rng: &mut R, _n: usize) -> Self {
        let len = (rng.random::<u8>() % 9) as usize;
        let mut items = Vec::with_capacity(len);
        for _ in 0..len {
            items.push(rng.random::<i32>());
        }
        ItemsInput { items }
    }
}

impl<R: Rng> CcArbitrary<R> for DrainInput {
    fn generate(rng: &mut R, _n: usize) -> Self {
        let len = (rng.random::<u8>() % 9) as usize;
        let mut items = Vec::with_capacity(len);
        for _ in 0..len {
            items.push(rng.random::<i32>());
        }
        let a = rng.random::<u8>() as usize;
        let b = rng.random::<u8>() as usize;
        DrainInput { items, a, b }
    }
}

// ============================================================================
// proptest strategies
// ============================================================================

fn items_strategy() -> BoxedStrategy<ItemsInput> {
    prop::collection::vec(any::<i32>(), 0..=8)
        .prop_map(|items| ItemsInput { items })
        .boxed()
}

fn drain_strategy() -> BoxedStrategy<DrainInput> {
    (
        prop::collection::vec(any::<i32>(), 0..=8),
        any::<u8>(),
        any::<u8>(),
    )
        .prop_map(|(items, a, b)| DrainInput {
            items,
            a: a as usize,
            b: b as usize,
        })
        .boxed()
}

// ============================================================================
// proptest adapter
// ============================================================================

fn run_proptest_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_proptest_property);
    }
    let counter = Arc::new(AtomicU64::new(0));
    let t0 = Instant::now();
    let cfg = ProptestConfig {
        cases: cases_budget().min(u32::MAX as u64) as u32,
        max_shrink_iters: 32,
        failure_persistence: None,
        ..ProptestConfig::default()
    };
    let mut runner = proptest::test_runner::TestRunner::new(cfg);
    let c = counter.clone();
    let result: Result<(), String> = match property {
        "ArrayvecDebugMatchesSlice" => runner
            .run(&items_strategy(), move |v| {
                c.fetch_add(1, Ordering::Relaxed);
                let cex = format!("({:?})", v);
                let out = std::panic::catch_unwind(AssertUnwindSafe(|| {
                    property_arrayvec_debug_matches_slice(v.items.clone())
                }));
                match out {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => Ok(()),
                    Ok(PropertyResult::Fail(_)) | Err(_) => Err(TestCaseError::fail(cex)),
                }
            })
            .map_err(|e| match e {
                TestError::Fail(reason, _) => reason.to_string(),
                other => other.to_string(),
            }),
        "RemovePastEndPanics" => runner
            .run(&items_strategy(), move |v| {
                c.fetch_add(1, Ordering::Relaxed);
                let cex = format!("({:?})", v);
                let out = std::panic::catch_unwind(AssertUnwindSafe(|| {
                    property_remove_past_end_panics(v.items.clone())
                }));
                match out {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => Ok(()),
                    Ok(PropertyResult::Fail(_)) | Err(_) => Err(TestCaseError::fail(cex)),
                }
            })
            .map_err(|e| match e {
                TestError::Fail(reason, _) => reason.to_string(),
                other => other.to_string(),
            }),
        "SwapRemoveLastReturnsTail" => runner
            .run(&items_strategy(), move |v| {
                c.fetch_add(1, Ordering::Relaxed);
                let cex = format!("({:?})", v);
                let out = std::panic::catch_unwind(AssertUnwindSafe(|| {
                    property_swap_remove_last_returns_tail(v.items.clone())
                }));
                match out {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => Ok(()),
                    Ok(PropertyResult::Fail(_)) | Err(_) => Err(TestCaseError::fail(cex)),
                }
            })
            .map_err(|e| match e {
                TestError::Fail(reason, _) => reason.to_string(),
                other => other.to_string(),
            }),
        "DrainMatchesSliceRange" => runner
            .run(&drain_strategy(), move |v| {
                c.fetch_add(1, Ordering::Relaxed);
                let cex = format!("({:?})", v);
                let out = std::panic::catch_unwind(AssertUnwindSafe(|| {
                    property_drain_matches_slice_range(v.items.clone(), v.a, v.b)
                }));
                match out {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => Ok(()),
                    Ok(PropertyResult::Fail(_)) | Err(_) => Err(TestCaseError::fail(cex)),
                }
            })
            .map_err(|e| match e {
                TestError::Fail(reason, _) => reason.to_string(),
                other => other.to_string(),
            }),
        _ => {
            return (
                Err(format!("Unknown property for proptest: {}", property)),
                Metrics::default(),
            );
        }
    };
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = counter.load(Ordering::Relaxed);
    (result, Metrics { inputs, elapsed_us })
}

// ============================================================================
// quickcheck adapter (fork with `etna` feature — fn-pointer API)
// ============================================================================

static QC_COUNTER: AtomicU64 = AtomicU64::new(0);

fn qc_arrayvec_debug_matches_slice(v: ItemsInput) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_arrayvec_debug_matches_slice(v.items) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn qc_remove_past_end_panics(v: ItemsInput) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_remove_past_end_panics(v.items) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn qc_swap_remove_last_returns_tail(v: ItemsInput) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_swap_remove_last_returns_tail(v.items) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn qc_drain_matches_slice_range(v: DrainInput) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_drain_matches_slice_range(v.items, v.a, v.b) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn run_quickcheck_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_quickcheck_property);
    }
    QC_COUNTER.store(0, Ordering::Relaxed);
    let t0 = Instant::now();
    let budget = cases_budget();
    let mut qc = QuickCheck::new()
        .tests(budget)
        .max_tests(budget.saturating_mul(4))
        .max_time(Duration::from_secs(86_400));
    let result = match property {
        "ArrayvecDebugMatchesSlice" => {
            qc.quicktest(qc_arrayvec_debug_matches_slice as fn(ItemsInput) -> TestResult)
        }
        "RemovePastEndPanics" => {
            qc.quicktest(qc_remove_past_end_panics as fn(ItemsInput) -> TestResult)
        }
        "SwapRemoveLastReturnsTail" => {
            qc.quicktest(qc_swap_remove_last_returns_tail as fn(ItemsInput) -> TestResult)
        }
        "DrainMatchesSliceRange" => {
            qc.quicktest(qc_drain_matches_slice_range as fn(DrainInput) -> TestResult)
        }
        _ => {
            return (
                Err(format!("Unknown property for quickcheck: {}", property)),
                Metrics::default(),
            );
        }
    };
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = QC_COUNTER.load(Ordering::Relaxed);
    let status = match result.status {
        ResultStatus::Finished => Ok(()),
        ResultStatus::Failed { arguments } => Err(format!("({})", arguments.join(" "))),
        ResultStatus::Aborted { err } => Err(format!("quickcheck aborted: {:?}", err)),
        ResultStatus::TimedOut => Err("quickcheck timed out".to_string()),
        ResultStatus::GaveUp => Err(format!(
            "quickcheck gave up after {} tests",
            result.n_tests_passed
        )),
    };
    (status, Metrics { inputs, elapsed_us })
}

// ============================================================================
// crabcheck adapter
// ============================================================================

static CC_COUNTER: AtomicU64 = AtomicU64::new(0);

fn cc_arrayvec_debug_matches_slice(v: ItemsInput) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_arrayvec_debug_matches_slice(v.items) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_remove_past_end_panics(v: ItemsInput) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_remove_past_end_panics(v.items) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_swap_remove_last_returns_tail(v: ItemsInput) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_swap_remove_last_returns_tail(v.items) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_drain_matches_slice_range(v: DrainInput) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_drain_matches_slice_range(v.items, v.a, v.b) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn run_crabcheck_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_crabcheck_property);
    }
    CC_COUNTER.store(0, Ordering::Relaxed);
    let t0 = Instant::now();
    let cfg = crabcheck_qc::Config {
        tests: cases_budget(),
    };
    let result = match property {
        "ArrayvecDebugMatchesSlice" => {
            crabcheck_qc::quickcheck_with_config(cfg, cc_arrayvec_debug_matches_slice)
        }
        "RemovePastEndPanics" => {
            crabcheck_qc::quickcheck_with_config(cfg, cc_remove_past_end_panics)
        }
        "SwapRemoveLastReturnsTail" => {
            crabcheck_qc::quickcheck_with_config(cfg, cc_swap_remove_last_returns_tail)
        }
        "DrainMatchesSliceRange" => {
            crabcheck_qc::quickcheck_with_config(cfg, cc_drain_matches_slice_range)
        }
        _ => {
            return (
                Err(format!("Unknown property for crabcheck: {}", property)),
                Metrics::default(),
            );
        }
    };
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = CC_COUNTER.load(Ordering::Relaxed);
    let status = match result.status {
        crabcheck_qc::ResultStatus::Finished => Ok(()),
        crabcheck_qc::ResultStatus::Failed { arguments } => {
            Err(format!("({})", arguments.join(" ")))
        }
        crabcheck_qc::ResultStatus::TimedOut => Err("crabcheck timed out".to_string()),
        crabcheck_qc::ResultStatus::GaveUp => Err(format!(
            "crabcheck gave up: passed={}, discarded={}",
            result.passed, result.discarded
        )),
        crabcheck_qc::ResultStatus::Aborted { error } => {
            Err(format!("crabcheck aborted: {}", error))
        }
    };
    (status, Metrics { inputs, elapsed_us })
}

// ============================================================================
// hegel adapter (real hegeltest 0.3.7 — panic-on-cex API)
// ============================================================================

static HG_COUNTER: AtomicU64 = AtomicU64::new(0);

fn hegel_settings() -> HegelSettings {
    HegelSettings::new()
        .test_cases(cases_budget())
        .suppress_health_check(HealthCheck::all())
}

fn hg_draw_u8(tc: &TestCase) -> u8 {
    let v = tc.draw(hgen::integers::<u32>().min_value(0).max_value(255));
    v as u8
}

fn hg_draw_i32(tc: &TestCase) -> i32 {
    tc.draw(hgen::integers::<i32>().min_value(i32::MIN).max_value(i32::MAX))
}

fn run_hegel_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_hegel_property);
    }
    HG_COUNTER.store(0, Ordering::Relaxed);
    let t0 = Instant::now();
    let settings = hegel_settings();
    let run_result = std::panic::catch_unwind(AssertUnwindSafe(|| match property {
        "ArrayvecDebugMatchesSlice" => {
            Hegel::new(|tc: TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let len = (hg_draw_u8(&tc) % 9) as usize;
                let mut items = Vec::with_capacity(len);
                for _ in 0..len {
                    items.push(hg_draw_i32(&tc));
                }
                let cex = format!("({:?})", items);
                let out = std::panic::catch_unwind(AssertUnwindSafe(|| {
                    property_arrayvec_debug_matches_slice(items.clone())
                }));
                match out {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => {}
                    Ok(PropertyResult::Fail(_)) | Err(_) => panic!("{}", cex),
                }
            })
            .settings(settings.clone())
            .run();
        }
        "RemovePastEndPanics" => {
            Hegel::new(|tc: TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let len = (hg_draw_u8(&tc) % 9) as usize;
                let mut items = Vec::with_capacity(len);
                for _ in 0..len {
                    items.push(hg_draw_i32(&tc));
                }
                let cex = format!("({:?})", items);
                let out = std::panic::catch_unwind(AssertUnwindSafe(|| {
                    property_remove_past_end_panics(items.clone())
                }));
                match out {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => {}
                    Ok(PropertyResult::Fail(_)) | Err(_) => panic!("{}", cex),
                }
            })
            .settings(settings.clone())
            .run();
        }
        "SwapRemoveLastReturnsTail" => {
            Hegel::new(|tc: TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let len = (hg_draw_u8(&tc) % 9) as usize;
                let mut items = Vec::with_capacity(len);
                for _ in 0..len {
                    items.push(hg_draw_i32(&tc));
                }
                let cex = format!("({:?})", items);
                let out = std::panic::catch_unwind(AssertUnwindSafe(|| {
                    property_swap_remove_last_returns_tail(items.clone())
                }));
                match out {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => {}
                    Ok(PropertyResult::Fail(_)) | Err(_) => panic!("{}", cex),
                }
            })
            .settings(settings.clone())
            .run();
        }
        "DrainMatchesSliceRange" => {
            Hegel::new(|tc: TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let len = (hg_draw_u8(&tc) % 9) as usize;
                let mut items = Vec::with_capacity(len);
                for _ in 0..len {
                    items.push(hg_draw_i32(&tc));
                }
                let a = hg_draw_u8(&tc) as usize;
                let b = hg_draw_u8(&tc) as usize;
                let cex = format!("({:?} {} {})", items, a, b);
                let out = std::panic::catch_unwind(AssertUnwindSafe(|| {
                    property_drain_matches_slice_range(items.clone(), a, b)
                }));
                match out {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => {}
                    Ok(PropertyResult::Fail(_)) | Err(_) => panic!("{}", cex),
                }
            })
            .settings(settings.clone())
            .run();
        }
        _ => panic!("__unknown_property:{}", property),
    }));
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = HG_COUNTER.load(Ordering::Relaxed);
    let metrics = Metrics { inputs, elapsed_us };
    let status = match run_result {
        Ok(()) => Ok(()),
        Err(e) => {
            let msg = panic_msg(e);
            if let Some(rest) = msg.strip_prefix("__unknown_property:") {
                return (
                    Err(format!("Unknown property for hegel: {}", rest)),
                    Metrics::default(),
                );
            }
            Err(msg
                .strip_prefix("Property test failed: ")
                .unwrap_or(&msg)
                .to_string())
        }
    };
    (status, metrics)
}

fn run(tool: &str, property: &str) -> Outcome {
    match tool {
        "etna" => run_etna_property(property),
        "proptest" => run_proptest_property(property),
        "quickcheck" => run_quickcheck_property(property),
        "crabcheck" => run_crabcheck_property(property),
        "hegel" => run_hegel_property(property),
        _ => (Err(format!("Unknown tool: {}", tool)), Metrics::default()),
    }
}

fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn emit_json(
    tool: &str,
    property: &str,
    status: &str,
    metrics: Metrics,
    counterexample: Option<&str>,
    error: Option<&str>,
) {
    let cex = counterexample.map_or("null".to_string(), json_str);
    let err = error.map_or("null".to_string(), json_str);
    println!(
        "{{\"status\":{},\"tests\":{},\"discards\":0,\"time\":{},\"counterexample\":{},\"error\":{},\"tool\":{},\"property\":{}}}",
        json_str(status),
        metrics.inputs,
        json_str(&format!("{}us", metrics.elapsed_us)),
        cex,
        err,
        json_str(tool),
        json_str(property),
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <tool> <property>", args[0]);
        eprintln!("Tools: etna | proptest | quickcheck | crabcheck | hegel");
        eprintln!(
            "Properties: ArrayvecDebugMatchesSlice | RemovePastEndPanics | SwapRemoveLastReturnsTail | DrainMatchesSliceRange | All"
        );
        std::process::exit(2);
    }
    let (tool, property) = (args[1].as_str(), args[2].as_str());

    // Silence library-under-test panic noise; frameworks catch panics
    // internally, but the default hook still prints to stderr.
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let caught = std::panic::catch_unwind(AssertUnwindSafe(|| run(tool, property)));
    std::panic::set_hook(previous_hook);

    let (result, metrics) = match caught {
        Ok(outcome) => outcome,
        Err(payload) => {
            emit_json(
                tool,
                property,
                "aborted",
                Metrics::default(),
                None,
                Some(&format!("adapter panic: {}", panic_msg(payload))),
            );
            return;
        }
    };

    match result {
        Ok(()) => emit_json(tool, property, "passed", metrics, None, None),
        Err(msg) => emit_json(tool, property, "failed", metrics, Some(&msg), None),
    }
}
