use std::collections::BTreeMap;

use bevy_ecs::system::Resource;

use crate::{ArcHandlerSystem, Event, HandlerConfig};

/// [`Resource`] which stores the registry of [`HandlerConfig`]s for a specific [`Event`] `E`,
/// sorted by priority.
#[derive(Resource)]
pub struct HandlerRegistry<E: Event> {
    handlers: BTreeMap<i32, Vec<HandlerConfig<E>>>,
}

impl<E: Event> HandlerRegistry<E> {
    /// Inserts a handler into the registry.
    pub fn insert(&mut self, config: HandlerConfig<E>) {
        self.handlers
            .entry(config.priority)
            .or_default()
            .push(config);
    }

    /// Returns an iterator over all handlers in the registry, from highest to lowest priority.
    pub fn handlers(&self) -> impl Iterator<Item = &ArcHandlerSystem<E>> {
        self.handlers.values().rev().flatten().map(|c| &c.handler)
    }
}

impl<E: Event> Default for HandlerRegistry<E> {
    fn default() -> Self {
        Self {
            handlers: BTreeMap::new(),
        }
    }
}
