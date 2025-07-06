use criterion::{Criterion, criterion_group, criterion_main};
use driveshaft::DriveShaftPool;
use std::{collections::VecDeque, sync::Arc};
use rocksdb::{DBWithThreadMode, SingleThreaded, DB};

use tokio::{
    runtime::Builder,
    task::spawn_blocking,
};

const N_TASKS: usize = 100_000;
const N_PUTS: usize = 1;
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

    let db = Arc::new(DB::open_default("./blocking").unwrap());

    c.bench_function("spawn_blocking io", |b| {
        b.to_async(&rt).iter(|| async {
            let mut tasks = Vec::new();
            for _ in 0..N_TASKS {
                let db = Arc::clone(&db);
                let task = tokio::spawn( async {
                    spawn_blocking(|| put_n_times(db, N_PUTS)).await
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

    let db = Arc::new(DB::open_default("./sync").unwrap());

    c.bench_function("no spawn io", |b| {
        b.to_async(&rt).iter(|| async {
            let mut tasks = Vec::new();
            for _ in 0..N_TASKS {
                let db = Arc::clone(&db);
                let task = tokio::spawn( async {
                    put_n_times(db, N_PUTS)
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

    let db = Arc::new(DB::open_default("./driveshaft").unwrap());

    let ctxs: VecDeque<Arc<_>> = (0..NUM_BLOCKING).map(|_| db.clone()).collect();
    let raw_pool = Arc::new(DriveShaftPool::new(ctxs));


    c.bench_function("driveshaft io", |b| {
        let raw_pool = raw_pool.clone();
        b.to_async(&rt).iter(|| async {
            let mut tasks = Vec::new();
            for _ in 0..N_TASKS {
                let raw_pool = raw_pool.clone();
                let task = tokio::spawn(async move{
                    raw_pool.run_with(|ctx| put_n_times_mut(ctx, N_PUTS)).await
                });
                tasks.push(task);
            }
            for task in tasks {
                let _ = task.await.unwrap();
            }
        });
    });
}


pub fn put_n_times(db: Arc<DBWithThreadMode<SingleThreaded>>, n: usize) {
    for _ in 0..n {
        db.put(b"{i}", b"{i}").unwrap();
    }
}

pub fn put_n_times_mut(db: &mut Arc<DBWithThreadMode<SingleThreaded>>, n: usize) {
    for _ in 0..n {
        db.put(b"{i}", b"{i}").unwrap();
    }
}


criterion_group!(benches, bench_spawn_blocking, bench_driveshaft, bench_sync_on_runtime);
criterion_main!(benches);
