//! Bounded concurrent execution utilities.
//!
//! Provides [`map_concurrent`] for running an async function over a collection
//! with a configurable concurrency limit, preserving input order.

use std::future::Future;
use std::sync::Arc;

use tokio::sync::Semaphore;

/// Applies an async function to each item with bounded concurrency, returning results in input order.
///
/// Spawns a tokio task per item, using a semaphore to limit how many run `f` concurrently.
/// Results are collected in the same order as the input items.
///
/// # Panics
///
/// - If `max_concurrency` is zero.
/// - If any spawned task panics.
pub async fn map_concurrent<T, R, F, Fut>(items: Vec<T>, max_concurrency: usize, f: F) -> Vec<R>
where
    T: Send + 'static,
    R: Send + 'static,
    F: Fn(T) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = R> + Send + 'static,
{
    assert!(max_concurrency > 0, "max_concurrency must be at least 1");

    if items.is_empty() {
        return Vec::new();
    }

    let semaphore = Arc::new(Semaphore::new(max_concurrency));
    let f = Arc::new(f);

    let handles: Vec<_> = items
        .into_iter()
        .map(|item| {
            let sem = semaphore.clone();
            let f = f.clone();
            tokio::spawn(async move {
                let _permit = sem.acquire().await.expect("semaphore closed unexpectedly");
                f(item).await
            })
        })
        .collect();

    let mut results = Vec::with_capacity(handles.len());
    for handle in handles {
        results.push(handle.await.expect("spawned task panicked"));
    }

    results
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    use super::map_concurrent;

    #[tokio::test]
    async fn empty_input_returns_empty_output() {
        let results: Vec<i32> =
            map_concurrent(Vec::<i32>::new(), 3, |item| async move { item * 2 }).await;

        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn preserves_input_order() {
        let items: Vec<i32> = (0..10).collect();

        let results = map_concurrent(items, 3, |item| async move { item * 2 }).await;

        assert_eq!(results, vec![0, 2, 4, 6, 8, 10, 12, 14, 16, 18]);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn respects_concurrency_limit() {
        let active = Arc::new(AtomicUsize::new(0));
        let max_observed = Arc::new(AtomicUsize::new(0));

        let items: Vec<i32> = (0..10).collect();

        let results = map_concurrent(items, 3, {
            let active = active.clone();
            let max_observed = max_observed.clone();
            move |item: i32| {
                let active = active.clone();
                let max_observed = max_observed.clone();
                async move {
                    let current = active.fetch_add(1, Ordering::SeqCst) + 1;
                    max_observed.fetch_max(current, Ordering::SeqCst);
                    tokio::time::sleep(Duration::from_millis(20)).await;
                    active.fetch_sub(1, Ordering::SeqCst);
                    item * 2
                }
            }
        })
        .await;

        assert_eq!(results, vec![0, 2, 4, 6, 8, 10, 12, 14, 16, 18]);
        assert!(max_observed.load(Ordering::SeqCst) <= 3);
    }

    #[tokio::test]
    async fn single_item_works() {
        let results = map_concurrent(vec![42], 5, |item| async move { item + 1 }).await;

        assert_eq!(results, vec![43]);
    }

    #[tokio::test]
    async fn concurrency_greater_than_items_works() {
        let results = map_concurrent(vec![1, 2, 3], 100, |item| async move { item * 3 }).await;

        assert_eq!(results, vec![3, 6, 9]);
    }

    #[tokio::test]
    #[should_panic(expected = "max_concurrency must be at least 1")]
    async fn zero_concurrency_panics() {
        let _ = map_concurrent(vec![1], 0, |item: i32| async move { item }).await;
    }
}
