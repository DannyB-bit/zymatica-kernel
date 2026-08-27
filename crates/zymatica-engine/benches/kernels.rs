use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use zymatica_core::ops::{matvec, matvec3};
use zymatica_core::quant::{
    QuantMatrix, RowQ3Matrix, RowQ4Matrix, RowQ5Matrix, RowQ8Matrix, quantize_activation_q8,
};
use zymatica_core::tensor::Matrix;

fn deterministic_matrix(rows: usize, cols: usize, phase: f32) -> Matrix {
    let data = (0..rows * cols)
        .map(|idx| {
            let x = idx as f32 + phase;
            (x.sin() * 0.75) + ((x * 0.125).cos() * 0.25)
        })
        .collect();
    Matrix::from_row_major(rows, cols, data)
}

fn deterministic_vector(cols: usize) -> Vec<f32> {
    (0..cols)
        .map(|idx| {
            let x = idx as f32;
            (x * 0.017).sin() + (x * 0.031).cos() * 0.5
        })
        .collect()
}

fn bench_quantized_matvecs(c: &mut Criterion) {
    let rows = 512;
    let cols = 768;
    let matrix = deterministic_matrix(rows, cols, 0.3);
    let x = deterministic_vector(cols);
    let q8 = RowQ8Matrix::quantize(&matrix);
    let q5 = RowQ5Matrix::quantize(&matrix);
    let q4 = RowQ4Matrix::quantize(&matrix);
    let q3 = RowQ3Matrix::quantize(&matrix);
    let (xq, x_scale) = quantize_activation_q8(&x);

    let mut group = c.benchmark_group("quantized_matvec");
    group.bench_function("f32_reference", |b| {
        b.iter(|| matvec(black_box(&matrix), black_box(&x)))
    });
    group.bench_function("q8_weight_f32_activation", |b| {
        b.iter(|| q8.matvec(black_box(&x)))
    });
    group.bench_function("q8_weight_q8_activation", |b| {
        b.iter(|| q8.matvec_q8_activation(black_box(&xq), black_box(x_scale)))
    });
    group.bench_function("q5_weight_f32_activation", |b| {
        b.iter(|| q5.matvec(black_box(&x)))
    });
    group.bench_function("q5_weight_q8_activation", |b| {
        b.iter(|| q5.matvec_q8_activation(black_box(&xq), black_box(x_scale)))
    });
    group.bench_function("q4_weight_f32_activation", |b| {
        b.iter(|| q4.matvec(black_box(&x)))
    });
    group.bench_function("q3_weight_f32_activation", |b| {
        b.iter(|| q3.matvec(black_box(&x)))
    });
    group.finish();
}

fn bench_fused_projection(c: &mut Criterion) {
    let q = deterministic_matrix(256, 768, 1.0);
    let k = deterministic_matrix(128, 768, 2.0);
    let v = deterministic_matrix(128, 768, 3.0);
    let x = deterministic_vector(768);
    let q8 = QuantMatrix::Q8Resident(RowQ8Matrix::quantize(&q));
    let k8 = QuantMatrix::Q8Resident(RowQ8Matrix::quantize(&k));
    let v8 = QuantMatrix::Q8Resident(RowQ8Matrix::quantize(&v));
    let q5 = QuantMatrix::Q5Resident(RowQ5Matrix::quantize(&q));
    let k5 = QuantMatrix::Q5Resident(RowQ5Matrix::quantize(&k));
    let v5 = QuantMatrix::Q5Resident(RowQ5Matrix::quantize(&v));

    let mut group = c.benchmark_group("fused_projection");
    group.bench_function("f32_qkv_matvec3", |b| {
        b.iter(|| matvec3(black_box(&q), black_box(&k), black_box(&v), black_box(&x)))
    });
    group.bench_function("q8_qkv_matvec3", |b| {
        b.iter(|| {
            QuantMatrix::matvec3(
                black_box(&q8),
                black_box(&k8),
                black_box(&v8),
                black_box(&x),
            )
        })
    });
    group.bench_function("q5_qkv_matvec3_q8_activation", |b| {
        b.iter(|| {
            QuantMatrix::matvec3_with_activation_mode(
                black_box(&q5),
                black_box(&k5),
                black_box(&v5),
                black_box(&x),
                zymatica_core::quant::QuantizedActivationMode::DynamicInt8,
            )
        })
    });
    group.finish();
}

fn bench_q3_fused_pair(c: &mut Criterion) {
    let rows = 512;
    let cols = 768;
    let a = RowQ3Matrix::quantize(&deterministic_matrix(rows, cols, 4.0));
    let b = RowQ3Matrix::quantize(&deterministic_matrix(rows, cols, 5.0));
    let x = deterministic_vector(cols);

    let mut group = c.benchmark_group("q3_fused_pair");
    group.bench_function("separate_matvecs", |bench| {
        bench.iter(|| {
            (
                black_box(&a).matvec(black_box(&x)),
                black_box(&b).matvec(black_box(&x)),
            )
        })
    });
    group.bench_function("fused_matvec2", |bench| {
        bench.iter(|| RowQ3Matrix::matvec2(black_box(&a), black_box(&b), black_box(&x)))
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_quantized_matvecs,
    bench_fused_projection,
    bench_q3_fused_pair
);
criterion_main!(benches);
