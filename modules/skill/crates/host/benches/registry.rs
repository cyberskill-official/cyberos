//! Compare hot-path registry access patterns under contention.
//!
//! Three implementations:
//!   1. Arc<DashMap>      — what we ship
//!   2. Arc<Mutex<HashMap>>  — naive baseline
//!   3. Arc<RwLock<HashMap>> — std::sync baseline
//!
//! Workload: N reader threads each doing M lookups against a pre-seeded
//! registry of K entries. Measures throughput (lookups/sec).

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use cyberos_skill_host::SkillRegistry;
use cyberos_skill_manifest::SkillManifest;
use parking_lot::Mutex;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

fn make_header(name: &str) -> cyberos_skill_host::SkillHeader {
    cyberos_skill_host::SkillHeader {
        manifest: SkillManifest {
            name: name.to_owned(),
            description: format!("benchmark header for {}", name),
            license: None,
            compatibility: None,
            metadata: Default::default(),
            allowed_tools: None,
            extra: Default::default(),
        },
        skill_dir: PathBuf::from("/tmp/bench"),
        body_offset: 0,
        file_size: 0,
    }
}

const SKILL_COUNT: usize = 64;
const LOOKUPS_PER_THREAD: usize = 10_000;

fn bench_dashmap(threads: usize) -> u64 {
    let reg = Arc::new(SkillRegistry::new());
    for i in 0..SKILL_COUNT {
        reg.insert_header(make_header(&format!("skill-{:03}", i)));
    }
    let mut handles = vec![];
    for t in 0..threads {
        let reg = Arc::clone(&reg);
        handles.push(thread::spawn(move || {
            let mut hits = 0u64;
            for i in 0..LOOKUPS_PER_THREAD {
                let name = format!("skill-{:03}", (t * 31 + i) % SKILL_COUNT);
                if reg.get_header(black_box(&name)).is_some() {
                    hits += 1;
                }
            }
            hits
        }));
    }
    handles.into_iter().map(|h| h.join().unwrap()).sum()
}

fn bench_mutex(threads: usize) -> u64 {
    type Reg = Arc<Mutex<HashMap<String, Arc<cyberos_skill_host::SkillHeader>>>>;
    let reg: Reg = Arc::new(Mutex::new(HashMap::new()));
    {
        let mut g = reg.lock();
        for i in 0..SKILL_COUNT {
            let h = make_header(&format!("skill-{:03}", i));
            g.insert(h.manifest.name.clone(), Arc::new(h));
        }
    }
    let mut handles = vec![];
    for t in 0..threads {
        let reg = Arc::clone(&reg);
        handles.push(thread::spawn(move || {
            let mut hits = 0u64;
            for i in 0..LOOKUPS_PER_THREAD {
                let name = format!("skill-{:03}", (t * 31 + i) % SKILL_COUNT);
                let g = reg.lock();
                if g.get(black_box(&name)).is_some() {
                    hits += 1;
                }
            }
            hits
        }));
    }
    handles.into_iter().map(|h| h.join().unwrap()).sum()
}

fn bench_rwlock(threads: usize) -> u64 {
    type Reg = Arc<RwLock<HashMap<String, Arc<cyberos_skill_host::SkillHeader>>>>;
    let reg: Reg = Arc::new(RwLock::new(HashMap::new()));
    {
        let mut g = reg.write();
        for i in 0..SKILL_COUNT {
            let h = make_header(&format!("skill-{:03}", i));
            g.insert(h.manifest.name.clone(), Arc::new(h));
        }
    }
    let mut handles = vec![];
    for t in 0..threads {
        let reg = Arc::clone(&reg);
        handles.push(thread::spawn(move || {
            let mut hits = 0u64;
            for i in 0..LOOKUPS_PER_THREAD {
                let name = format!("skill-{:03}", (t * 31 + i) % SKILL_COUNT);
                let g = reg.read();
                if g.get(black_box(&name)).is_some() {
                    hits += 1;
                }
            }
            hits
        }));
    }
    handles.into_iter().map(|h| h.join().unwrap()).sum()
}

fn bench_registry(c: &mut Criterion) {
    let mut group = c.benchmark_group("registry_lookup_contention");
    group.sample_size(20);
    for threads in [1usize, 4, 16, 64].iter() {
        group.bench_with_input(BenchmarkId::new("dashmap", threads), threads,
            |b, &t| b.iter(|| bench_dashmap(t)));
        group.bench_with_input(BenchmarkId::new("mutex", threads), threads,
            |b, &t| b.iter(|| bench_mutex(t)));
        group.bench_with_input(BenchmarkId::new("rwlock", threads), threads,
            |b, &t| b.iter(|| bench_rwlock(t)));
    }
    group.finish();
}

criterion_group!(benches, bench_registry);
criterion_main!(benches);
