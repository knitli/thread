use std::path::PathBuf;
use thread_flow::incremental::graph::DependencyGraph;
use thread_flow::incremental::types::{DependencyEdge, DependencyType};
use thread_utilities::RapidSet;

use std::hint::black_box;
use criterion::{Criterion, criterion_group, criterion_main};

fn bench_find_affected_files(c: &mut Criterion) {
    let mut graph = DependencyGraph::new();
    let num_files = 10000;
    let deps_per_file = 10;

    // Create nodes
    for i in 0..num_files {
        graph.add_node(&PathBuf::from(format!("file_{}.rs", i)));
    }

    // Create edges (linear chain with some random deps)
    for i in 0..num_files {
        for j in 1..=deps_per_file {
            let dep_idx = (i + j) % num_files;
            graph.add_edge(DependencyEdge::new(
                PathBuf::from(format!("file_{}.rs", i)),
                PathBuf::from(format!("file_{}.rs", dep_idx)),
                DependencyType::Import,
            ));
        }
    }

    let changed_files: RapidSet<PathBuf> = (0..10)
        .map(|i| PathBuf::from(format!("file_{}.rs", i)))
        .collect();

    c.bench_function("find_affected_files_10000_nodes", |b| {
        b.iter(|| {
            let _affected = graph.find_affected_files(black_box(&changed_files));
        })
    });
}

criterion_group!(benches, bench_find_affected_files);
criterion_main!(benches);
