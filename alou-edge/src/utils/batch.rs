#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;
use crate::utils::async_lock::{RwLock, Mutex};
use std::future::Future;
use crate::utils::error::Result;

/// Request batcher for deduplicating and batching concurrent requests
/// 
/// This helps optimize performance by:
/// 1. Deduplicating identical concurrent requests
/// 2. Batching multiple requests together
/// 3. Reducing load on backend services
pub struct RequestBatcher<K, V> {
    pending: Arc<RwLock<HashMap<K, Arc<Mutex<Option<Result<V>>>>>>>,
}

impl<K, V> RequestBatcher<K, V>
where
    K: std::hash::Hash + Eq + Clone,
    V: Clone,
{
    pub fn new() -> Self {
        Self {
            pending: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Execute a request with deduplication
    /// 
    /// If the same key is requested concurrently, only one execution happens
    /// and all callers receive the same result.
    pub async fn execute<F, Fut>(&self, key: K, f: F) -> Result<V>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<V>>,
    {
        // Check if request is already pending
        let result_lock = {
            let mut pending = self.pending.write().await;
            
            if let Some(existing) = pending.get(&key) {
                existing.clone()
            } else {
                let new_lock = Arc::new(Mutex::new(None));
                pending.insert(key.clone(), new_lock.clone());
                new_lock
            }
        };
        
        // Try to acquire the lock
        let mut result_guard = result_lock.lock().await;
        
        // If result is already computed, return it
        if let Some(result) = result_guard.as_ref() {
            return result.clone();
        }
        
        // Execute the function
        let result = f().await;
        
        // Store the result (clone for storage)
        let stored_result = match &result {
            Ok(v) => Ok(v.clone()),
            Err(e) => Err(e.clone()),
        };
        *result_guard = Some(stored_result);
        
        // Clean up from pending map
        {
            let mut pending = self.pending.write().await;
            pending.remove(&key);
        }
        
        result
    }
}

impl<K, V> Default for RequestBatcher<K, V>
where
    K: std::hash::Hash + Eq + Clone,
    V: Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    #[tokio::test]
    async fn test_request_deduplication() {
        let batcher = RequestBatcher::<String, String>::new();
        let counter = Arc::new(AtomicUsize::new(0));
        
        let counter1 = counter.clone();
        let counter2 = counter.clone();
        
        // Launch two concurrent requests with the same key
        let handle1 = tokio::spawn({
            let batcher = batcher.clone();
            async move {
                batcher.execute("test".to_string(), || async {
                    counter1.fetch_add(1, Ordering::SeqCst);
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    Ok("result".to_string())
                }).await
            }
        });
        
        let handle2 = tokio::spawn({
            let batcher = batcher.clone();
            async move {
                batcher.execute("test".to_string(), || async {
                    counter2.fetch_add(1, Ordering::SeqCst);
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    Ok("result".to_string())
                }).await
            }
        });
        
        let result1 = handle1.await.unwrap().unwrap();
        let result2 = handle2.await.unwrap().unwrap();
        
        // Both should get the same result
        assert_eq!(result1, result2);
        
        // Function should only be called once
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }
}

impl<K, V> Clone for RequestBatcher<K, V> {
    fn clone(&self) -> Self {
        Self {
            pending: self.pending.clone(),
        }
    }
}
