/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Independently driven Tokio runtimes for partition-placement tests.

use aws_smithy_http_client::pool::DriverSpawner;
use std::future::Future;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Arc};
use std::thread::{self, JoinHandle};
use tokio::runtime::{Handle, Id};
use tokio::sync::oneshot;

/// A current-thread Tokio runtime driven by its own operating-system thread.
#[derive(Debug)]
pub(crate) struct DrivenRuntime {
    handle: Handle,
    runtime_id: Id,
    shutdown: Option<oneshot::Sender<()>>,
    thread: Option<JoinHandle<()>>,
    submitted_tasks: Arc<AtomicUsize>,
}

impl DrivenRuntime {
    /// Starts a current-thread runtime and waits until its handle is available.
    pub(crate) fn start(name: &str) -> Self {
        let (ready_tx, ready_rx) = mpsc::sync_channel(1);
        let (shutdown, shutdown_rx) = oneshot::channel();
        let submitted_tasks = Arc::new(AtomicUsize::new(0));
        let thread = thread::Builder::new()
            .name(name.to_owned())
            .spawn(move || {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("test runtime should build");
                let handle = runtime.handle().clone();
                let runtime_id = handle.id();
                ready_tx
                    .send((handle, runtime_id))
                    .expect("runtime owner should receive its handle");
                runtime.block_on(async {
                    let _ = shutdown_rx.await;
                });
            })
            .expect("test runtime thread should start");
        let (handle, runtime_id) = ready_rx
            .recv()
            .expect("test runtime should publish its handle");
        Self {
            handle,
            runtime_id,
            shutdown: Some(shutdown),
            thread: Some(thread),
            submitted_tasks,
        }
    }

    /// Returns a spawner that records task placement on this runtime.
    pub(crate) fn driver_spawner(&self) -> DriverSpawner {
        let handle = self.handle.clone();
        let submitted_tasks = self.submitted_tasks.clone();
        DriverSpawner::from_fn(move |driver| {
            submitted_tasks.fetch_add(1, Ordering::SeqCst);
            drop(handle.spawn(driver));
        })
    }

    /// Spawns test orchestration work on this runtime.
    pub(crate) fn spawn<F>(&self, future: F) -> tokio::task::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.handle.spawn(future)
    }

    /// Returns the runtime's stable Tokio identity.
    pub(crate) fn id(&self) -> Id {
        self.runtime_id
    }

    /// Returns how many connection-owned tasks were submitted to this runtime.
    pub(crate) fn submitted_tasks(&self) -> usize {
        self.submitted_tasks.load(Ordering::SeqCst)
    }

    /// Stops the runtime, drops all runtime-owned tasks, and joins its thread.
    pub(crate) fn shutdown(mut self) {
        self.stop();
    }

    fn stop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        if let Some(thread) = self.thread.take() {
            thread.join().expect("test runtime thread should not panic");
        }
    }
}

impl Drop for DrivenRuntime {
    fn drop(&mut self) {
        self.stop();
    }
}
