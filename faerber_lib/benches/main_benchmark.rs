use criterion::{criterion_group, criterion_main, Criterion};

use faerber_lib::{
    convert_color, convert_dither, convert_naive, rgba_pixels_to_labs, ConversionMethod,
};
use image::RgbaImage;

// ty NyxKrage <3
macro_rules! include_root_bytes {
    ($path: expr) => {
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/", $path))
    };
}

pub fn benchmark(c: &mut Criterion) {
    // benchmark image: Wanderer über dem Nebelmeer - by Casper David Friedrich
    let img: RgbaImage = image::load_from_memory(include_root_bytes!(
        "../assets/tests/wanderer-ueber-dem-nebelmeer.jpg"
    ))
    .unwrap()
    .to_rgba8();

    // benchmark colorscheme: Nord - by Arctic Ice Studio
    let colors: &[u32] = &[
        0xB58DAE, 0xA2BF8A, 0xECCC87, 0xD2876D, 0xC16069, 0x5D80AE, 0x80A0C2, 0x86C0D1, 0x8EBCBB,
        0xECEFF4, 0xE5E9F0, 0xD8DEE9, 0x4C566B, 0x434C5F, 0x3B4253, 0x2E3440,
    ];

    let random_pixel = img
        .pixels()
        .nth(rand::random::<usize>() % img.pixels().count())
        .expect("Image should have at least one pixel");
    let random_lab = faerber_lib::Lab::from_rgba(&random_pixel.0);
    let palette = faerber_lib::convert_palette_to_lab(colors);

    c.benchmark_group("pixel")
        .sample_size(100)
        .bench_function("de1976", |b| {
            b.iter(|| convert_color(ConversionMethod::De1976, &palette, &random_lab))
        })
        .bench_function("de1994g", |b| {
            b.iter(|| convert_color(ConversionMethod::De1994G, &palette, &random_lab))
        })
        .bench_function("de1994t", |b| {
            b.iter(|| convert_color(ConversionMethod::De1994T, &palette, &random_lab))
        })
        .bench_function("de2000", |b| {
            b.iter(|| convert_color(ConversionMethod::De2000, &palette, &random_lab))
        });

    c.benchmark_group("image")
        .sample_size(10)
        .bench_function("de1976", |b| {
            b.iter(|| convert_naive(&img, ConversionMethod::De1976, &palette))
        })
        .bench_function("de1994g", |b| {
            b.iter(|| convert_naive(&img, ConversionMethod::De1994G, &palette))
        })
        .bench_function("de1994t", |b| {
            b.iter(|| convert_naive(&img, ConversionMethod::De1994T, &palette))
        })
        .bench_function("de2000", |b| {
            b.iter(|| convert_naive(&img, ConversionMethod::De2000, &palette))
        })
        .bench_function("dither_sierra3", |b| {
            b.iter(|| convert_dither(&img, ConversionMethod::DitherSierra3, &colors))
        });

    c.benchmark_group("other")
        .sample_size(10)
        .bench_function("rgba_pixels_to_lab", |b| {
            b.iter(|| rgba_pixels_to_labs(img.pixels().clone()))
        });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
