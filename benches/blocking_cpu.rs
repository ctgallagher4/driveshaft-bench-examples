use criterion::{Criterion, criterion_group, criterion_main};
use driveshaft::DriveShaftPool;
use std::{collections::VecDeque, sync::Arc};
use sha2::{Digest, Sha256};
use tokio::{
    runtime::Builder,
    task::spawn_blocking,
};

const N_TASKS: usize = 100_000;
const N_HASHES: usize = 10;
const NUM_WORKERS: usize = 1;
const NUM_BLOCKING: usize = 4;
const NUM_UNUSED: usize = 1;

fn bench_spawn_blocking(c: &mut Criterion) {
    let rt = Builder::new_multi_thread()
        .worker_threads(NUM_WORKERS)
        .max_blocking_threads(NUM_BLOCKING)
        .enable_all()
        .build()
        .unwrap();


    c.bench_function("spawn_blocking cpu intense", |b| {
        b.to_async(&rt).iter(|| async {
            let mut tasks = Vec::new();
            for _ in 0..N_TASKS {
                let task = tokio::spawn( async {
                    spawn_blocking(|| hash_n_times(b"hash this", N_HASHES)).await
                });
                tasks.push(task);
            }
            for task in tasks {
                let _ = task.await.unwrap();
            }
        });
    });
}

fn bench_sync_on_runtime(c: &mut Criterion) {
    let rt = Builder::new_multi_thread()
        .worker_threads(NUM_WORKERS)
        .max_blocking_threads(NUM_UNUSED)
        .enable_all()
        .build()
        .unwrap();

    c.bench_function("no spawn cpu intense", |b| {
        b.to_async(&rt).iter(|| async {
            let mut tasks = Vec::new();
            for _ in 0..N_TASKS {
                let task = tokio::spawn( async {
                    hash_n_times(b"hash this", N_HASHES)
                });
                tasks.push(task);
            }
            for task in tasks {
                let _ = task.await.unwrap();
            }
        });
    });
}

fn bench_driveshaft(c: &mut Criterion) {
    let rt = Builder::new_multi_thread()
        .worker_threads(NUM_WORKERS)
        .max_blocking_threads(NUM_UNUSED)
        .enable_all()
        .build()
        .unwrap();

    let ctxs: VecDeque<u64> = (0..NUM_BLOCKING as u64).map(|i| i).collect();
    let raw_pool = Arc::new(DriveShaftPool::new(ctxs));

    c.bench_function("driveshaft cpu intense", |b| {
        let raw_pool = raw_pool.clone();
        b.to_async(&rt).iter(|| async {
            let mut tasks = Vec::new();
            for _ in 0..N_TASKS {
                let raw_pool = raw_pool.clone();
                let task = tokio::spawn(async move{
                    raw_pool.run_with(|_ctx| hash_n_times(b"hash this", N_HASHES)).await
                });
                tasks.push(task);
            }
            for task in tasks {
                let _ = task.await.unwrap();
            }
        });
    });
}

pub fn hash_n_times(data: &[u8], n: usize) -> Vec<u8> {
    let mut hash = data.to_vec();
    for _ in 0..n {
        let mut hasher = Sha256::new();
        hasher.update(&hash);
        hash = hasher.finalize().to_vec();
    }
    hash
}

criterion_group!(benches, bench_spawn_blocking, bench_driveshaft, bench_sync_on_runtime);
criterion_main!(benches);
