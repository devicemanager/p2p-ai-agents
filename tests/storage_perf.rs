use p2p_ai_agents::storage::local::{LocalStorage, DistributedStorage, CacheStorage, CustomStorage, Storage};
use std::time::Instant;

#[tokio::test]
async fn compare_storage_layer_performance() {
    let n = 10_000;
    let local = LocalStorage::new();
    let distributed = DistributedStorage::new();
    let cache = CacheStorage::new();
    let custom = CustomStorage::new();

    async fn run_perf_test<S: Storage>(storage: &S, label: &str, n: usize) {
        let start = Instant::now();
        for i in 0..n {
            let key = format!("key_{}", i);
            let value = vec![i as u8; 128];
            storage.put(&key, value).await.unwrap();
        }
        let duration = start.elapsed();
        println!("{}: Wrote {} keys in {:?}", label, n, duration);
    }

    run_perf_test(&local, "LocalStorage", n).await;
    run_perf_test(&distributed, "DistributedStorage (stub)", n).await;
    run_perf_test(&cache, "CacheStorage (stub)", n).await;
    run_perf_test(&custom, "CustomStorage (stub)", n).await;
}
