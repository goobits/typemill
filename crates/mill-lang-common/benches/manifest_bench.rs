use criterion::{criterion_group, criterion_main, Criterion};
use mill_lang_common::TomlWorkspace;
use std::hint::black_box;
use toml_edit::{DocumentMut, Item, Table};

// Inefficient cloning version (simulation of old behavior)
fn merge_dependencies_cloning(base: &str, source: &str) -> String {
    let mut base_doc = base.parse::<DocumentMut>().unwrap();
    let source_doc = source.parse::<DocumentMut>().unwrap();

    if let Some(item) = source_doc.get("dependencies") {
        if let Item::Table(source_table) = item {
            let base_deps = base_doc
                .entry("dependencies")
                .or_insert(Item::Table(Table::new()));

            if let Some(base_table) = base_deps.as_table_mut() {
                for (key, value) in source_table.iter() {
                    if !base_table.contains_key(key) {
                        base_table.insert(key, value.clone());
                    }
                }
            }
        }
    }
    base_doc.to_string()
}

// Optimized entry version (potential future behavior)
fn merge_dependencies_entry(base: &str, source: &str) -> String {
    let mut base_doc = base.parse::<DocumentMut>().unwrap();
    let mut source_doc = source.parse::<DocumentMut>().unwrap();

    // Consume source dependencies
    if let Some(item) = source_doc.remove("dependencies") {
        if let Item::Table(source_table) = item {
            let base_deps = base_doc
                .entry("dependencies")
                .or_insert(Item::Table(Table::new()));

            if let Some(base_table) = base_deps.as_table_mut() {
                for (key, value) in source_table.into_iter() {
                    base_table.entry(&key).or_insert(value);
                }
            }
        }
    }
    base_doc.to_string()
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("manifest_merge");

    // Create large TOMLs with complex values
    let mut base_toml = String::from("[dependencies]\n");
    for i in 0..100 {
        base_toml.push_str(&format!("dep{} = {{ version = \"1.0.0\", features = [\"extra\", \"stuff\"] }}\n", i));
    }

    let mut source_toml = String::from("[dependencies]\n");
    for i in 50..150 { // Overlap 50-99, new 100-149
        source_toml.push_str(&format!("dep{} = {{ version = \"2.0.0\", features = [\"new\", \"features\", \"here\"] }}\n", i));
    }

    group.bench_function("cloning (inefficient)", |b| {
        b.iter(|| merge_dependencies_cloning(black_box(&base_toml), black_box(&source_toml)))
    });

    group.bench_function("moving (current)", |b| {
        b.iter(|| TomlWorkspace::merge_dependencies(black_box(&base_toml), black_box(&source_toml)).unwrap())
    });

    group.bench_function("entry (optimized)", |b| {
        b.iter(|| merge_dependencies_entry(black_box(&base_toml), black_box(&source_toml)))
    });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
