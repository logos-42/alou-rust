use worker::*;
use serde::{Serialize, Deserialize};
use crate::utils::error::{AloudError, Result};

/// Cache TTL constants for different data types
#[allow(dead_code)]
pub mod cache_ttl {
    /// Session data: 24 hours
    pub const SESSION: u64 = 86400;
    /// Nonce: 5 minutes
    pub const NONCE: u64 = 300;
    /// RPC query results: 30 seconds
    pub const RPC_QUERY: u64 = 30;
    /// Token balance: 1 minute
    pub const TOKEN_BALANCE: u64 = 60;
    /// NFT metadata: 1 hour
    pub const NFT_METADATA: u64 = 3600;
    /// Contract ABI: 24 hours
    pub const CONTRACT_ABI: u64 = 86400;
}

/// Wrapper for KV Store operations with caching strategies
#[derive(Clone)]
pub struct KvStore {
    kv: kv::KvStore,
}

impl KvStore {
    pub fn new(kv: kv::KvStore) -> Self {
        Self { kv }
    }
    
    /// Get a value from KV store
    pub async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        match self.kv.get(key).json::<T>().await {
            Ok(value) => Ok(value),
            Err(e) => Err(AloudError::CacheError(format!("Failed to get key {}: {}", key, e))),
        }
    }
    
    /// Get a string value from KV store
    #[allow(dead_code)]
    pub async fn get_string(&self, key: &str) -> Result<Option<String>> {
        match self.kv.get(key).text().await {
            Ok(value) => Ok(value),
            Err(e) => Err(AloudError::CacheError(format!("Failed to get key {}: {}", key, e))),
        }
    }
    
    /// Put a value into KV store
    pub async fn put<T>(&self, key: &str, value: &T, ttl_seconds: Option<u64>) -> Result<()>
    where
        T: Serialize,
    {
        let json = serde_json::to_string(value)
            .map_err(|e| AloudError::CacheError(format!("Failed to serialize value: {}", e)))?;
        
        let builder = {
            let mut b = self.kv.put(key, json)
                .map_err(|e| AloudError::CacheError(format!("Failed to create put builder: {}", e)))?;
            
            if let Some(ttl) = ttl_seconds {
                b = b.expiration_ttl(ttl);
            }
            b
        };
        
        builder.execute().await
            .map_err(|e| AloudError::CacheError(format!("Failed to put key {}: {}", key, e)))?;
        
        Ok(())
    }
    
    /// Put a string value into KV store
    #[allow(dead_code)]
    pub async fn put_string(&self, key: &str, value: &str, ttl_seconds: Option<u64>) -> Result<()> {
        let builder = {
            let mut b = self.kv.put(key, value)
                .map_err(|e| AloudError::CacheError(format!("Failed to create put builder: {}", e)))?;
            
            if let Some(ttl) = ttl_seconds {
                b = b.expiration_ttl(ttl);
            }
            b
        };
        
        builder.execute().await
            .map_err(|e| AloudError::CacheError(format!("Failed to put key {}: {}", key, e)))?;
        
        Ok(())
    }
    
    /// Delete a value from KV store
    pub async fn delete(&self, key: &str) -> Result<()> {
        self.kv.delete(key).await
            .map_err(|e| AloudError::CacheError(format!("Failed to delete key {}: {}", key, e)))?;
        
        Ok(())
    }
    
    /// Check if a key exists in KV store
    #[allow(dead_code)]
    pub async fn exists(&self, key: &str) -> Result<bool> {
        match self.kv.get(key).text().await {
            Ok(Some(_)) => Ok(true),
            Ok(None) => Ok(false),
            Err(e) => Err(AloudError::CacheError(format!("Failed to check key {}: {}", key, e))),
        }
    }
    
    /// List keys with a prefix
    #[allow(dead_code)]
    pub async fn list(&self, prefix: &str, limit: Option<u64>) -> Result<Vec<String>> {
        let list_builder = {
            let mut b = self.kv.list();
            
            if !prefix.is_empty() {
                b = b.prefix(prefix.to_string());
            }
            
            if let Some(limit) = limit {
                b = b.limit(limit);
            }
            b
        };
        
        let result = list_builder.execute().await
            .map_err(|e| AloudError::CacheError(format!("Failed to list keys: {}", e)))?;
        
        Ok(result.keys.into_iter().map(|k| k.name).collect())
    }
    
    /// Get with cache-aside pattern: try cache first, then fallback
    pub async fn get_or_compute<T, F, Fut>(
        &self,
        key: &str,
        ttl_seconds: u64,
        compute: F,
    ) -> Result<T>
    where
        T: Serialize + for<'de> Deserialize<'de>,
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        // Try to get from cache first
        if let Some(cached) = self.get::<T>(key).await? {
            return Ok(cached);
        }
        
        // Cache miss - compute value
        let value = compute().await?;
        
        // Store in cache (fire and forget - don't fail if cache write fails)
        let _ = self.put(key, &value, Some(ttl_seconds)).await;
        
        Ok(value)
    }
    
    /// Batch get operation for multiple keys
    pub async fn get_batch<T>(&self, keys: &[String]) -> Result<Vec<Option<T>>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut results = Vec::with_capacity(keys.len());
        
        for key in keys {
            results.push(self.get::<T>(key).await?);
        }
        
        Ok(results)
    }
    
    /// Invalidate cache by prefix (useful for clearing related data)
    pub async fn invalidate_prefix(&self, prefix: &str) -> Result<usize> {
        let keys = self.list(prefix, Some(1000)).await?;
        let count = keys.len();
        
        for key in keys {
            let _ = self.delete(&key).await; // Continue on error
        }
        
        Ok(count)
    }
}
