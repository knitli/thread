// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

use async_trait::async_trait;
use cocoindex::ops::interface::TargetFactory;
use std::future::Future;

/// Strategy pattern for handling runtime environment differences
/// (CLI/Local vs Cloudflare/Edge)
#[async_trait]
pub trait RuntimeStrategy: Send + Sync {
    /// Spawn a future in the environment's preferred way
    fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static;

    // Abstract other environment specifics (storage, config, etc.)
}

pub struct LocalStrategy;

#[async_trait]
impl RuntimeStrategy for LocalStrategy {
    fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        tokio::spawn(future);
    }
}

pub struct EdgeStrategy;

#[async_trait]
impl RuntimeStrategy for EdgeStrategy {
    fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        // Cloudflare Workers specific spawning if needed, or generic tokio
        tokio::spawn(future);
    }
}
