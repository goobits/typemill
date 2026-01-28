use criterion::{criterion_group, criterion_main, Criterion};
use mill_lang_common::manifest_common::TomlWorkspace;
use std::hint::black_box;
use toml_edit::{DocumentMut, Item, Table};

fn generate_large_toml(dep_count: usize) -> String {
    let mut s = String::new();
    s.push_str("[package]\nname = \"test\"\nversion = \"0.1.0\"\n\n");

    s.push_str("[dependencies]\n");
    for i in 0..dep_count {
        s.push_str(&format!("dep{} = \"1.0.{}\"\n", i, i));
    }

    s.push_str("\n[dev-dependencies]\n");
    for i in 0..dep_count {
        s.push_str(&format!("dev-dep{} = \"1.0.{}\"\n", i, i));
    }
    s
}

// Inefficient implementation (simulating the "before" state)
fn merge_dependencies_cloning(base: &str, source: &str) -> String {
    let mut base_doc = base.parse::<DocumentMut>().unwrap();
    let source_doc = source.parse::<DocumentMut>().unwrap();

    // Handle dependencies
    if let Some(source_deps) = source_doc.get("dependencies").and_then(|i| i.as_table()) {
        let base_deps = base_doc
            .entry("dependencies")
            .or_insert(Item::Table(Table::new()));

        if let Some(base_table) = base_deps.as_table_mut() {
            for (key, value) in source_deps.iter() {
                if !base_table.contains_key(key) {
                    base_table.insert(key, value.clone());
                }
            }
        }
    }

    // Handle dev-dependencies
    if let Some(source_deps) = source_doc.get("dev-dependencies").and_then(|i| i.as_table()) {
        let base_deps = base_doc
            .entry("dev-dependencies")
            .or_insert(Item::Table(Table::new()));

        if let Some(base_table) = base_deps.as_table_mut() {
            for (key, value) in source_deps.iter() {
                if !base_table.contains_key(key) {
                    base_table.insert(key, value.clone());
                }
            }
        }
    }
    base_doc.to_string()
}

fn bench_merge(c: &mut Criterion) {
    // Generate large TOMLs
    // 500 dependencies is reasonably large to show impact of allocation/cloning
    let base_toml = generate_large_toml(10);
    let source_toml = generate_large_toml(500);

    let mut group = c.benchmark_group("merge_dependencies");

    group.bench_function("optimized (current)", |b| {
        b.iter(|| {
            TomlWorkspace::merge_dependencies(
                black_box(&base_toml),
                black_box(&source_toml)
            ).unwrap()
        })
    });

    group.bench_function("inefficient (cloning)", |b| {
        b.iter(|| {
            merge_dependencies_cloning(
                black_box(&base_toml),
                black_box(&source_toml)
            )
        })
    });

    group.finish();
}

criterion_group!(benches, bench_merge);
criterion_main!(benches);
