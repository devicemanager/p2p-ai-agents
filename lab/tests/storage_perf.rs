use p2p_ai_agents::storage::local::{
    CacheStorage, CustomStorage, DistributedStorage, LocalStorage, Storage,
};
#[cfg(feature = "storage-supabase")]
use p2p_ai_agents::storage::supabase::{SupabaseConfig, SupabaseStorage};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub struct PerfResults {
    pub backend: String,
    pub operations: usize,
    pub duration: Duration,
    pub ops_per_second: f64,
    pub avg_latency_micros: f64,
}

impl PerfResults {
    pub fn new(backend: String, operations: usize, duration: Duration) -> Self {
        let ops_per_second = operations as f64 / duration.as_secs_f64();
        let avg_latency_micros = duration.as_micros() as f64 / operations as f64;

        Self {
            backend,
            operations,
            duration,
            ops_per_second,
            avg_latency_micros,
        }
    }

    pub fn print_summary(&self) {
        println!(
            "{:20} | {:>8} ops | {:>8.2}ms | {:>10.2} ops/s | {:>8.2}μs avg",
            self.backend,
            self.operations,
            self.duration.as_millis(),
            self.ops_per_second,
            self.avg_latency_micros
        );
    }
}

/// Performance test suite for storage adapters
pub struct StoragePerfTest {
    pub operations: usize,
    pub data_size: usize,
    pub concurrency: usize,
}

impl Default for StoragePerfTest {
    fn default() -> Self {
        Self {
            operations: 1000,
            data_size: 1024, // 1KB
            concurrency: 10,
        }
    }
}

impl StoragePerfTest {
    pub fn new(operations: usize, data_size: usize, concurrency: usize) -> Self {
        Self {
            operations,
            data_size,
            concurrency,
        }
    }

    /// Run sequential write test
    pub async fn run_write_test<S: Storage>(&self, storage: &S, label: &str) -> PerfResults {
        let start = Instant::now();
        let test_data = vec![42u8; self.data_size];

        for i in 0..self.operations {
            let key = format!("perf_write_{}", i);
            storage.put(&key, test_data.clone()).await.unwrap();
        }

        let duration = start.elapsed();
        PerfResults::new(format!("{} (write)", label), self.operations, duration)
    }

    /// Run sequential read test
    pub async fn run_read_test<S: Storage>(&self, storage: &S, label: &str) -> PerfResults {
        // First populate data
        let test_data = vec![42u8; self.data_size];
        for i in 0..self.operations {
            let key = format!("perf_read_{}", i);
            storage.put(&key, test_data.clone()).await.unwrap();
        }

        // Now measure read performance
        let start = Instant::now();
        for i in 0..self.operations {
            let key = format!("perf_read_{}", i);
            let _data = storage.get(&key).await.unwrap();
        }

        let duration = start.elapsed();
        PerfResults::new(format!("{} (read)", label), self.operations, duration)
    }

    /// Run concurrent read/write test
    pub async fn run_concurrent_test<S: Storage + Send + Sync + 'static>(
        &self,
        storage: Arc<S>,
        label: &str,
    ) -> PerfResults {
        let start = Instant::now();
        let mut handles = Vec::new();
        let ops_per_task = self.operations / self.concurrency;
        let test_data = vec![42u8; self.data_size];

        for task_id in 0..self.concurrency {
            let storage = storage.clone();
            let test_data = test_data.clone();
            let handle = tokio::spawn(async move {
                for i in 0..ops_per_task {
                    let key = format!("perf_concurrent_{}_{}", task_id, i);
                    // Mix of writes and reads
                    if i % 2 == 0 {
                        storage.put(&key, test_data.clone()).await.unwrap();
                    } else {
                        let read_key = format!("perf_concurrent_{}_{}", task_id, i - 1);
                        let _data = storage.get(&read_key).await.unwrap();
                    }
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }

        let duration = start.elapsed();
        let total_ops = self.concurrency * ops_per_task;
        PerfResults::new(format!("{} (concurrent)", label), total_ops, duration)
    }

    /// Run delete test
    pub async fn run_delete_test<S: Storage>(&self, storage: &S, label: &str) -> PerfResults {
        // First populate data
        let test_data = vec![42u8; self.data_size];
        for i in 0..self.operations {
            let key = format!("perf_delete_{}", i);
            storage.put(&key, test_data.clone()).await.unwrap();
        }

        // Now measure delete performance
        let start = Instant::now();
        for i in 0..self.operations {
            let key = format!("perf_delete_{}", i);
            storage.delete(&key).await.unwrap();
        }

        let duration = start.elapsed();
        PerfResults::new(format!("{} (delete)", label), self.operations, duration)
    }

    /// Run comprehensive benchmark suite
    pub async fn run_comprehensive_benchmark(&self) -> Vec<PerfResults> {
        let mut results = Vec::new();

        // Test LocalStorage
        let local_storage = LocalStorage::new();
        results.push(self.run_write_test(&local_storage, "LocalStorage").await);
        results.push(self.run_read_test(&local_storage, "LocalStorage").await);
        results.push(self.run_delete_test(&local_storage, "LocalStorage").await);
        results.push(
            self.run_concurrent_test(Arc::new(local_storage), "LocalStorage")
                .await,
        );

        // Test DistributedStorage (stub)
        let distributed_storage = DistributedStorage::new();
        results.push(
            self.run_write_test(&distributed_storage, "DistributedStorage")
                .await,
        );
        results.push(
            self.run_read_test(&distributed_storage, "DistributedStorage")
                .await,
        );
        results.push(
            self.run_delete_test(&distributed_storage, "DistributedStorage")
                .await,
        );

        // Test CacheStorage (stub)
        let cache_storage = CacheStorage::new();
        results.push(self.run_write_test(&cache_storage, "CacheStorage").await);
        results.push(self.run_read_test(&cache_storage, "CacheStorage").await);
        results.push(self.run_delete_test(&cache_storage, "CacheStorage").await);

        // Test CustomStorage (stub)
        let custom_storage = CustomStorage::new();
        results.push(self.run_write_test(&custom_storage, "CustomStorage").await);
        results.push(self.run_read_test(&custom_storage, "CustomStorage").await);
        results.push(self.run_delete_test(&custom_storage, "CustomStorage").await);

        #[cfg(feature = "storage-supabase")]
        {
            // Test SupabaseStorage (if configured)
            if let Ok(supabase_storage) = self.create_test_supabase_storage().await {
                println!("Testing Supabase storage adapter...");
                results.push(
                    self.run_write_test(&supabase_storage, "SupabaseStorage")
                        .await,
                );
                results.push(
                    self.run_read_test(&supabase_storage, "SupabaseStorage")
                        .await,
                );
                results.push(
                    self.run_delete_test(&supabase_storage, "SupabaseStorage")
                        .await,
                );
                results.push(
                    self.run_concurrent_test(Arc::new(supabase_storage), "SupabaseStorage")
                        .await,
                );
            } else {
                println!("Skipping Supabase tests - no valid configuration found");
            }
        }

        results
    }

    #[cfg(feature = "storage-supabase")]
    async fn create_test_supabase_storage(&self) -> Result<SupabaseStorage, Box<dyn std::error::Error>> {
        // Try to get config from environment variables
        let url = std::env::var("SUPABASE_URL").unwrap_or_else(|_| "https://demo.supabase.co".to_string());
        let anon_key = std::env::var("SUPABASE_ANON_KEY").unwrap_or_else(|_| "test-key".to_string());
        
        let config = SupabaseConfig {
            url,
            anon_key,
            service_role_key: std::env::var("SUPABASE_SERVICE_ROLE_KEY").ok(),
            schema: std::env::var("SUPABASE_SCHEMA").unwrap_or_else(|_| "public".to_string()),
            table_name: std::env::var("SUPABASE_TABLE_NAME").unwrap_or_else(|_| "storage_perf_test".to_string()),
            timeout: 30,
            max_retries: 3,
        };

        SupabaseStorage::new(config)
    }

    /// Print benchmark results in a nice table format
    pub fn print_results(results: &[PerfResults]) {
        println!("\n🚀 Storage Performance Benchmark Results");
        println!("========================================================");
        println!(
            "{:20} | {:>8} | {:>8} | {:>10} | {:>8}",
            "Backend", "Ops", "Time", "Ops/Sec", "Avg Lat"
        );
        println!("========================================================");

        for result in results {
            result.print_summary();
        }

        println!("========================================================");

        // Find best performers
        let mut write_results: Vec<_> = results.iter().filter(|r| r.backend.contains("write")).collect();
        let mut read_results: Vec<_> = results.iter().filter(|r| r.backend.contains("read")).collect();
        let mut delete_results: Vec<_> = results.iter().filter(|r| r.backend.contains("delete")).collect();
        let mut concurrent_results: Vec<_> = results.iter().filter(|r| r.backend.contains("concurrent")).collect();

        write_results.sort_by(|a, b| b.ops_per_second.partial_cmp(&a.ops_per_second).unwrap());
        read_results.sort_by(|a, b| b.ops_per_second.partial_cmp(&a.ops_per_second).unwrap());
        delete_results.sort_by(|a, b| b.ops_per_second.partial_cmp(&a.ops_per_second).unwrap());
        concurrent_results.sort_by(|a, b| b.ops_per_second.partial_cmp(&a.ops_per_second).unwrap());

        println!("\n🏆 Performance Leaders:");
        if let Some(best_write) = write_results.first() {
            println!("  Write:      {}", best_write.backend);
        }
        if let Some(best_read) = read_results.first() {
            println!("  Read:       {}", best_read.backend);
        }
        if let Some(best_delete) = delete_results.first() {
            println!("  Delete:     {}", best_delete.backend);
        }
        if let Some(best_concurrent) = concurrent_results.first() {
            println!("  Concurrent: {}", best_concurrent.backend);
        }
    }
}

#[tokio::test]
async fn test_storage_performance_small() {
    let test = StoragePerfTest::new(100, 256, 4); // Small test
    let results = test.run_comprehensive_benchmark().await;
    StoragePerfTest::print_results(&results);
    
    // Verify all tests ran
    assert!(!results.is_empty());
    for result in &results {
        assert!(result.operations > 0);
        assert!(result.duration.as_nanos() > 0);
    }
}

#[tokio::test]
async fn test_storage_performance_medium() {
    let test = StoragePerfTest::new(1000, 1024, 10); // Medium test
    let results = test.run_comprehensive_benchmark().await;
    StoragePerfTest::print_results(&results);
    
    // Verify all tests ran
    assert!(!results.is_empty());
    for result in &results {
        assert!(result.operations > 0);
        assert!(result.duration.as_nanos() > 0);
    }
}

#[tokio::test]
#[ignore] // Only run when explicitly requested
async fn test_storage_performance_large() {
    let test = StoragePerfTest::new(10000, 4096, 20); // Large test
    let results = test.run_comprehensive_benchmark().await;
    StoragePerfTest::print_results(&results);
}

#[tokio::test]
async fn test_local_storage_stress() {
    let storage = Arc::new(LocalStorage::new());
    let mut handles = Vec::new();
    let operations_per_task = 1000;
    let num_tasks = 10;

    // Launch concurrent tasks
    for task_id in 0..num_tasks {
        let storage = storage.clone();
        let handle = tokio::spawn(async move {
            for i in 0..operations_per_task {
                let key = format!("stress_{}_{}", task_id, i);
                let value = vec![task_id as u8; 128];
                
                // Put
                storage.put(&key, value.clone()).await.unwrap();
                
                // Get and verify
                let retrieved = storage.get(&key).await.unwrap();
                assert_eq!(retrieved, Some(value));
                
                // Small delay to allow other tasks
                if i % 100 == 0 {
                    sleep(Duration::from_millis(1)).await;
                }
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify final state
    for task_id in 0..num_tasks {
        for i in 0..operations_per_task {
            let key = format!("stress_{}_{}", task_id, i);
            let value = storage.get(&key).await.unwrap();
            assert_eq!(value, Some(vec![task_id as u8; 128]));
        }
    }
}

#[cfg(feature = "storage-supabase")]
#[tokio::test]
async fn test_supabase_storage_performance() {
    // This test requires proper Supabase configuration
    let config = SupabaseConfig {
        url: std::env::var("SUPABASE_URL").unwrap_or_else(|_| {
            println!("SUPABASE_URL not set, skipping test");
            return;
        }),
        anon_key: std::env::var("SUPABASE_ANON_KEY").unwrap_or_else(|_| {
            println!("SUPABASE_ANON_KEY not set, skipping test");
            return;
        }),
        service_role_key: std::env::var("SUPABASE_SERVICE_ROLE_KEY").ok(),
        schema: "public".to_string(),
        table_name: "storage_perf_test".to_string(),
        timeout: 30,
        max_retries: 3,
    };

    if let Ok(storage) = SupabaseStorage::new(config) {
        let test = StoragePerfTest::new(100, 1024, 4);
        
        println!("Testing Supabase storage performance...");
        let write_result = test.run_write_test(&storage, "Supabase").await;
        let read_result = test.run_read_test(&storage, "Supabase").await;
        let delete_result = test.run_delete_test(&storage, "Supabase").await;
        
        println!("\nSupabase Storage Performance Results:");
        write_result.print_summary();
        read_result.print_summary();
        delete_result.print_summary();
        
        // Verify reasonable performance
        assert!(write_result.ops_per_second > 0.0);
        assert!(read_result.ops_per_second > 0.0);
        assert!(delete_result.ops_per_second > 0.0);
    } else {
        println!("Could not create Supabase storage - check configuration");
    }
}
