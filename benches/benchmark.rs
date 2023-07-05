use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, black_box};
use rustor::model::actor::ActorPool;

fn create_large_graph(n: usize) -> ActorPool {
    let total_actors = n;

    let pool = ActorPool::new();
    let mut actor_ids: Vec<usize> = Vec::new();

    for _ in 0..total_actors {
        let actor_id = pool.create_actor();
        actor_ids.push(actor_id);
    }

    for i in 1..total_actors {
        pool.subscribe(actor_ids[i], vec![actor_ids[i - 1]])
            .unwrap();
    }

    // create cycle by making the last actor subscribe to the first actor
    pool.subscribe(actor_ids[0], vec![actor_ids[total_actors - 1]])
        .unwrap();

    pool
}

fn create_sparse_subscribe_with_no_loop_graph() {
    let total_actors = 1000;

    let pool = ActorPool::new();
    let mut actor_ids: Vec<usize> = Vec::new();

    for _ in 0..total_actors {
        let actor_id = pool.create_actor();
        actor_ids.push(actor_id);
    }

    for i in 1..total_actors {
        pool.subscribe(actor_ids[i], vec![actor_ids[i - 1]])
            .unwrap();
    }
}

fn benchmark(c: &mut Criterion) {
    let pool = create_large_graph(1500);

    c.bench_function("dfs_cycle_detection", |f| {
        f.iter(|| pool.detect_cycle_dfs(0).unwrap())
    });

    c.bench_function("bfs_cycle_detection", |f| {
        f.iter(|| pool.detect_cycle_bfs(0).unwrap())
    });

    c.bench_function("topological_sort_cycle_detection", |f| {
        f.iter(|| pool.detect_cycle_topological_sort(0).unwrap())
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);