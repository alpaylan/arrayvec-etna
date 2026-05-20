//! Fault-localization integration tests for arrayvec.
//!
//! One `#[test]` per property in src/bin/etna-faultloc.rs's dispatch.
//! Each test runs `crabcheck::quickcheck_with_locate!` on the property,
//! prints the report, and emits a single `@@LOCATE@@ {<json>}` line on
//! stdout. Tests never panic — the driver classifies success/failure
//! from the JSON.

#![cfg(feature = "etna")]

use arrayvec::etna::{
    property_extend_panics_on_overflow, property_insert_at_length_succeeds, PropertyResult,
};
use crabcheck::quickcheck::{Arbitrary, Mutate};
use rand_etna::Rng;

#[derive(Clone, Debug)]
struct ExtendInput {
    items: Vec<u32>,
}

#[derive(Clone, Debug)]
struct InsertInput {
    existing: Vec<u32>,
    value: u32,
}

impl<R: Rng> Arbitrary<R> for ExtendInput {
    fn generate(rng: &mut R, _n: usize) -> Self {
        let len = (rng.random::<u8>() % 9) as usize;
        ExtendInput {
            items: (0..len).map(|_| rng.random::<u32>()).collect(),
        }
    }
}

impl<R: Rng> Arbitrary<R> for InsertInput {
    fn generate(rng: &mut R, _n: usize) -> Self {
        let len = (rng.random::<u8>() % 8) as usize;
        InsertInput {
            existing: (0..len).map(|_| rng.random::<u32>()).collect(),
            value: rng.random::<u32>(),
        }
    }
}

impl<R: Rng> Mutate<R> for ExtendInput {
    fn mutate(&self, rng: &mut R, _n: usize) -> Self {
        let mut out = self.clone();
        match rng.random_range(0u8..3) {
            0 if !out.items.is_empty() => {
                let i = rng.random_range(0..out.items.len());
                let b = rng.random_range(0u32..32);
                out.items[i] ^= 1u32 << b;
            }
            1 if out.items.len() < 12 => out.items.push(rng.random::<u32>()),
            _ if !out.items.is_empty() => {
                out.items.pop();
            }
            _ => {}
        }
        out
    }
}

impl<R: Rng> Mutate<R> for InsertInput {
    fn mutate(&self, rng: &mut R, _n: usize) -> Self {
        let mut out = self.clone();
        match rng.random_range(0u8..4) {
            0 => {
                let b = rng.random_range(0u32..32);
                out.value ^= 1u32 << b;
            }
            1 if !out.existing.is_empty() => {
                let i = rng.random_range(0..out.existing.len());
                let b = rng.random_range(0u32..32);
                out.existing[i] ^= 1u32 << b;
            }
            2 if out.existing.len() < 12 => out.existing.push(rng.random::<u32>()),
            _ if !out.existing.is_empty() => {
                out.existing.pop();
            }
            _ => {}
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

fn property_extend_panics_on_overflow_test(i: ExtendInput) -> Option<bool> {
    to_opt(property_extend_panics_on_overflow(i.items))
}

fn property_insert_at_length_succeeds_test(i: InsertInput) -> Option<bool> {
    to_opt(property_insert_at_length_succeeds(i.existing, i.value))
}

// Manual JSON emitter (we don't depend on serde_json in dev-deps).
fn json_escape(s: &str) -> String {
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

fn json_f64(x: f64) -> String {
    if x.is_finite() {
        format!("{}", x)
    } else {
        "null".to_string()
    }
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
        format!(
            "{{\"rank\":{},\"file\":{},\"function\":{},\"start_line\":{},\"end_line\":{},\"ochiai\":{},\"delta\":{},\"panic_overlap\":{},\"confidence\":{},\"confidence_rule\":{}}}",
            s.rank,
            json_escape(&s.region.file),
            json_escape(&s.region.function),
            s.region.start_line,
            s.region.end_line,
            json_f64(s.region.suspiciousness.ochiai as f64),
            json_f64(s.region.delta as f64),
            s.panic_overlap,
            json_escape(&format!("{}", s.confidence)),
            json_escape(s.confidence_rule),
        )
    } else {
        "null".to_string()
    };
    let top_5_items: Vec<String> = r
        .suspects
        .iter()
        .take(5)
        .map(|s| {
            format!(
                "{{\"rank\":{},\"file\":{},\"function\":{},\"start_line\":{},\"end_line\":{},\"confidence\":{},\"confidence_rule\":{},\"panic_overlap\":{}}}",
                s.rank,
                json_escape(&s.region.file),
                json_escape(&s.region.function),
                s.region.start_line,
                s.region.end_line,
                json_escape(&format!("{}", s.confidence)),
                json_escape(s.confidence_rule),
                s.panic_overlap,
            )
        })
        .collect();
    let top_5 = format!("[{}]", top_5_items.join(","));
    let diag_items: Vec<String> = r.diagnostics.iter().map(|d| json_escape(d.tag())).collect();
    let diags = format!("[{}]", diag_items.join(","));
    let out = format!(
        "{{\"status\":{},\"passed\":{},\"discarded\":{},\"n_panics\":{},\"n_suspects\":{},\"top\":{},\"top_5\":{},\"diagnostics\":{}}}",
        json_escape(status),
        r.run.passed,
        r.run.discarded,
        r.n_panics,
        r.suspects.len(),
        top,
        top_5,
        diags,
    );
    println!("@@LOCATE@@ {}", out);
}

#[test]
fn locate_extend_panics_on_overflow() {
    let report =
        crabcheck::quickcheck_with_locate!(property_extend_panics_on_overflow_test, "arrayvec");
    eprintln!("{report}");
    emit_locate_json(&report);
}

#[test]
fn locate_insert_at_length_succeeds() {
    let report =
        crabcheck::quickcheck_with_locate!(property_insert_at_length_succeeds_test, "arrayvec");
    eprintln!("{report}");
    emit_locate_json(&report);
}
