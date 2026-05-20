// Crabcheck fault-localization runner for arrayvec.
use std::fmt;

use arrayvec::etna::{
    property_extend_panics_on_overflow, property_insert_at_length_succeeds, PropertyResult,
};
use crabcheck::profiling::quickcheck;
use crabcheck::quickcheck::{Arbitrary, Mutate};
use rand_etna::Rng;

#[derive(Clone)]
struct ExtendInput { items: Vec<u32> }
impl fmt::Debug for ExtendInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{:?}", self.items) }
}

#[derive(Clone)]
struct InsertInput { existing: Vec<u32>, value: u32 }
impl fmt::Debug for InsertInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} {}", self.existing, self.value)
    }
}

impl<R: Rng> Arbitrary<R> for ExtendInput {
    fn generate(rng: &mut R, _n: usize) -> Self {
        let len = (rng.random::<u8>() % 9) as usize;
        ExtendInput { items: (0..len).map(|_| rng.random::<u32>()).collect() }
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
            },
            1 if out.items.len() < 12 => out.items.push(rng.random::<u32>()),
            _ if !out.items.is_empty() => { out.items.pop(); },
            _ => {},
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
            },
            1 if !out.existing.is_empty() => {
                let i = rng.random_range(0..out.existing.len());
                let b = rng.random_range(0u32..32);
                out.existing[i] ^= 1u32 << b;
            },
            2 if out.existing.len() < 12 => out.existing.push(rng.random::<u32>()),
            _ if !out.existing.is_empty() => { out.existing.pop(); },
            _ => {},
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
    if args.len() < 3 {
        eprintln!("Usage: {} <tool> <property> [tests]", args[0]);
        return;
    }
    let tool = args[1].as_str();
    let property = args[2].as_str();
    let result = match (tool, property) {
        ("crabcheck", "ExtendPanicsOnOverflow") => {
            quickcheck(|i: ExtendInput| {
                to_opt(property_extend_panics_on_overflow(i.items))
            })
        },
        ("crabcheck", "InsertAtLengthSucceeds") => {
            quickcheck(|i: InsertInput| {
                to_opt(property_insert_at_length_succeeds(i.existing, i.value))
            })
        },
        _ => panic!("Unknown: {tool} {property}"),
    };
    println!("Result: {:?}", result);
}
