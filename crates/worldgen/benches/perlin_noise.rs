use criterion::{criterion_group, criterion_main, Criterion};
use spherix_math::vector::{Vector2f, Vector3f};
use spherix_worldgen::noise::perlin::octave::MultiOctaveNoiseFactory;
use spherix_worldgen::noise::perlin::simplex::MultiOctaveNoise as SimplexMultiOctaveNoise;
use spherix_worldgen::noise::perlin::{GridNoise, MultiOctaveNoise, Noise, SimplexNoise};
use spherix_worldgen::rng::XoroShiro;
use std::hint::black_box;

pub fn single_octave(c: &mut Criterion) {
    c.bench_function("single_octave_grid_noise", |b| {
        let noise = GridNoise::new(&mut XoroShiro::new(0xAA78B4));
        let mut i = 0;

        b.iter(|| {
            black_box(noise.sample(vector(i)));

            i += 1;
        })
    });

    c.bench_function("single_octave_simplex_noise", |b| {
        let noise = SimplexNoise::new(&mut XoroShiro::new(0xAA78B4));
        let mut i = 0;

        b.iter(|| {
            black_box(noise.sample(vector(i)));

            i += 1;
        })
    });
}

pub fn multi_octave(c: &mut Criterion) {
    c.bench_function("multi_octave_grid_noise", |b| {
        let noise = MultiOctaveNoise::create(
            &mut XoroShiro::new(0xAA78B4),
            &vec![
                1.0,
                1.0,
                1.0,
                1.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.013333333333333334
            ],
            -8
        );

        let mut i = 0;

        b.iter(|| {
            black_box(noise.sample(vector(i)));

            i += 1;
        })
    });

    c.bench_function("multi_octave_simplex_noise", |b| {
        let noise = SimplexMultiOctaveNoise::create(
            &mut XoroShiro::new(0xAA78B4),
            &vec![
                1.0,
                1.0,
                1.0,
                1.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.013333333333333334
            ],
            -8
        );

        let mut i = 0;

        b.iter(|| {
            black_box(noise.sample(Vector2f::new(25.0, i as f64)));

            i += 1;
        })
    });
}

#[inline]
fn vector(i: i32) -> Vector3f {
    Vector3f::new(49.5, i as f64, -102.18)
}

criterion_group!(benches, single_octave, multi_octave);
criterion_main!(benches);
