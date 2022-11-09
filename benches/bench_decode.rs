#[macro_use]
extern crate criterion;

use ckb_vm::instructions::{b, i, v, m};
use ckb_vm_decoder::{rvb_decoder, rvi_decoder, rvv_decoder, rvm_decoder};
use criterion::{BenchmarkId, Criterion};
use rand::prelude::*;

macro_rules! bench_rand_decode {
    ($name:literal, $hash_based:ident, $pattern_based:ident) => {
        pub fn $hash_based(c: &mut Criterion) {
            let gen_inst = |idx: usize| -> u32 {
                let i = idx % $hash_based::INSTRUCTION_LIST.len();
                $hash_based::INSTRUCTION_LIST[i].get_match_bits()
            };

            const ITERATION: usize = 100_000_000;
            let mut group = c.benchmark_group("group");
            group.sample_size(10);

            let seed = rand::thread_rng().gen();
            let mut rng0 = SmallRng::from_seed(seed);
            let mut rng1 = SmallRng::from_seed(seed);

            group.bench_function(BenchmarkId::new(concat!($name, "-pattern-based"), 0), |b| {
                b.iter(|| {
                    for _ in 0..ITERATION {
                        let idx: usize = rng0.gen();
                        let bits = gen_inst(idx);
                        $pattern_based::factory::<u64>(bits, 0);
                    }
                })
            });

            group.bench_function(BenchmarkId::new(concat!($name, "-hash-based"), 0), |b| {
                b.iter(|| {
                    for _ in 0..ITERATION {
                        let idx: usize = rng1.gen();
                        let bits = gen_inst(idx);
                        $hash_based::factory::<u64>(bits, 0);
                    }
                })
            });
            group.finish();
        }
    };
}

macro_rules! bench_groups {
    ($( ($name: literal, $hash_based:ident, $pattern_based:ident )),*) => {
        $(
            bench_rand_decode!($name, $hash_based, $pattern_based);
        )*
        criterion_group!(
            benches,
            $( $hash_based),*
        );
    }
}

bench_groups!(
    ("rvv", rvv_decoder, v),
    ("rvb", rvb_decoder, b),
    ("rvi", rvi_decoder, i),
    ("rvm", rvm_decoder, m)
);
criterion_main!(benches);
