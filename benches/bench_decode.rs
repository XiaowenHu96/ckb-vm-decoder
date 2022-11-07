#[macro_use]
extern crate criterion;

use ckb_vm::instructions::{b, v};
use ckb_vm_decoder::{rvb_decoder, rvv_decoder};
use criterion::{BenchmarkId, Criterion};
use rand::prelude::*;

pub fn bench_rvv_decode(c: &mut Criterion) {
    let gen_inst = |idx: usize| -> u32 {
        let i = idx % rvv_decoder::INSTRUCTION_LIST.len();
        rvv_decoder::INSTRUCTION_LIST[i].get_match_bits()
    };

    const ITERATION: usize = 100_000_000;
    let mut group = c.benchmark_group("group");
    group.sample_size(10);

    let seed = rand::thread_rng().gen();
    let mut rng0 = SmallRng::from_seed(seed);
    let mut rng1 = SmallRng::from_seed(seed);

    group.bench_function(BenchmarkId::new("rvv-match-based", 0), |b| {
        b.iter(|| {
            for _ in 0..ITERATION {
                let idx: usize = rng0.gen();
                let bits = gen_inst(idx);
                v::factory::<u64>(bits, 0);
            }
        })
    });

    group.bench_function(BenchmarkId::new("rvv-hash-based", 0), |b| {
        b.iter(|| {
            for _ in 0..ITERATION {
                let idx: usize = rng1.gen();
                let bits = gen_inst(idx);
                rvv_decoder::factory::<u64>(bits, 0);
            }
        })
    });

    group.finish();
}

pub fn bench_rvb_decode(c: &mut Criterion) {
    let gen_inst = |idx: usize| -> u32 {
        let i = idx % rvb_decoder::INSTRUCTION_LIST.len();
        rvb_decoder::INSTRUCTION_LIST[i].get_match_bits()
    };

    const ITERATION: usize = 100_000_000;
    let mut group = c.benchmark_group("group");
    group.sample_size(10);

    let seed = rand::thread_rng().gen();
    let mut rng0 = SmallRng::from_seed(seed);
    let mut rng1 = SmallRng::from_seed(seed);

    group.bench_function(BenchmarkId::new("rvb-match-based", 0), |b| {
        b.iter(|| {
            for _ in 0..ITERATION {
                let idx: usize = rng0.gen();
                let bits = gen_inst(idx);
                b::factory::<u64>(bits, 0);
            }
        })
    });

    group.bench_function(BenchmarkId::new("rvb-hash-based", 0), |b| {
        b.iter(|| {
            for _ in 0..ITERATION {
                let idx: usize = rng1.gen();
                let bits = gen_inst(idx);
                rvb_decoder::factory::<u64>(bits, 0);
            }
        })
    });

    group.finish();
}

criterion_group!(benches, bench_rvv_decode, bench_rvb_decode);
criterion_main!(benches);
