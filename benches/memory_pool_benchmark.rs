use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dingo_test_runner::util::memory_pool::{get_string_vec, get_row_data, get_byte_vec};

fn benchmark_string_vec_allocation(c: &mut Criterion) {
    c.bench_function("string_vec_standard", |b| {
        b.iter(|| {
            let mut vec: Vec<String> = Vec::new();
            for i in 0..100 {
                vec.push(format!("test_{}", i));
            }
            black_box(vec)
        })
    });

    c.bench_function("string_vec_pooled", |b| {
        b.iter(|| {
            let mut vec = get_string_vec();
            for i in 0..100 {
                vec.push(format!("test_{}", i));
            }
            black_box(vec)
        })
    });
}

fn benchmark_row_data_allocation(c: &mut Criterion) {
    c.bench_function("row_data_standard", |b| {
        b.iter(|| {
            let mut rows: Vec<Vec<String>> = Vec::new();
            for i in 0..50 {
                let mut row = Vec::new();
                for j in 0..10 {
                    row.push(format!("cell_{}_{}", i, j));
                }
                rows.push(row);
            }
            black_box(rows)
        })
    });

    c.bench_function("row_data_pooled", |b| {
        b.iter(|| {
            let mut rows = get_row_data();
            for i in 0..50 {
                let mut row = get_string_vec();
                for j in 0..10 {
                    row.push(format!("cell_{}_{}", i, j));
                }
                rows.push(row.take());
            }
            black_box(rows)
        })
    });
}

fn benchmark_byte_vec_allocation(c: &mut Criterion) {
    c.bench_function("byte_vec_standard", |b| {
        b.iter(|| {
            let mut vec: Vec<u8> = Vec::new();
            for i in 0..1000 {
                vec.push((i % 256) as u8);
            }
            black_box(vec)
        })
    });

    c.bench_function("byte_vec_pooled", |b| {
        b.iter(|| {
            let mut vec = get_byte_vec();
            for i in 0..1000 {
                vec.push((i % 256) as u8);
            }
            black_box(vec)
        })
    });
}

criterion_group!(
    benches,
    benchmark_string_vec_allocation,
    benchmark_row_data_allocation,
    benchmark_byte_vec_allocation
);
criterion_main!(benches);