use std::sync::Arc;

use parking_lot::Mutex;

use crate::{ArcHandlerSystem, Event, IntoHandlerSystem};

/// Configuration for an event handler.
///
/// # Priority
///
/// Handlers are ran in order of priority, with higher priority handlers being ran first.
/// Individual handlers can be assigned a priority using the [`HandlerConfig::priority`] method.
///
/// Handlers with the same priority are ran in the order they were added.
pub struct HandlerConfig<E: Event> {
    pub(crate) priority: i32,
    pub(crate) handler: ArcHandlerSystem<E, ()>,
}

impl<E: Event> HandlerConfig<E> {
    /// Creates a new handler configuration.
    pub fn new(handler: ArcHandlerSystem<E, ()>) -> Self {
        Self {
            priority: HandlerPriority::priority(&Normal),
            handler,
        }
    }

    /// Sets the priority of the handler.
    pub fn priority(mut self, priority: impl HandlerPriority) -> Self {
        self.priority = HandlerPriority::priority(&priority);
        self
    }
}

/// Trait for types that can be converted into a [`HandlerConfig`].
pub trait IntoHandlerConfig<E: Event, Marker>: Sized {
    /// Converts the type into a [`HandlerConfig`].
    fn into_config(self) -> HandlerConfig<E>;

    /// Sets the priority of the handler.
    fn priority(self, priority: impl HandlerPriority) -> HandlerConfig<E> {
        self.into_config().priority(priority)
    }
}

/// [`HandlerConfig`]s can be converted into themselves.
impl<E: Event> IntoHandlerConfig<E, ()> for HandlerConfig<E> {
    fn into_config(self) -> HandlerConfig<E> {
        self
    }
}

/// [`ArcHandlerSystem`]s can be converted into [`HandlerConfig`]s.
impl<E: Event> IntoHandlerConfig<E, ()> for ArcHandlerSystem<E, ()> {
    fn into_config(self) -> HandlerConfig<E> {
        HandlerConfig::new(self)
    }
}

#[doc(hidden)]
pub struct SystemMarker;

/// Anything that can be converted into a [`HandlerSystem`] can be converted into a [`HandlerConfig`].
///
/// [`HandlerSystem`]: crate::HandlerSystem
impl<E: Event, Marker, S: IntoHandlerSystem<E, (), Marker>>
    IntoHandlerConfig<E, (SystemMarker, Marker)> for S
{
    fn into_config(self) -> HandlerConfig<E> {
        let system = Arc::new(Mutex::new(IntoHandlerSystem::into_system(self)));
        HandlerConfig::new(system)
    }
}

/// Trait for types that can be converted into a priority value.
pub trait HandlerPriority {
    /// Higher priority handlers are ran first.
    fn priority(&self) -> i32;
}

/// [`i32`] can be converted into a priority value.
impl HandlerPriority for i32 {
    fn priority(&self) -> i32 {
        *self
    }
}

/// [`HandlerPriority`] that runs with first priority.
pub struct First;

impl HandlerPriority for First {
    fn priority(&self) -> i32 {
        i32::MAX
    }
}

/// [`HandlerPriority`] that runs with early priority.
pub struct Early;

impl HandlerPriority for Early {
    fn priority(&self) -> i32 {
        i32::MAX / 2
    }
}

/// [`HandlerPriority`] that runs with pre priority.
pub struct Pre;

impl HandlerPriority for Pre {
    fn priority(&self) -> i32 {
        i32::MAX / 4
    }
}

/// [`HandlerPriority`] that runs with normal priority.
pub struct Normal;

impl HandlerPriority for Normal {
    fn priority(&self) -> i32 {
        0
    }
}

/// [`HandlerPriority`] that runs with post priority.
pub struct Post;

impl HandlerPriority for Post {
    fn priority(&self) -> i32 {
        i32::MIN / 4
    }
}

/// [`HandlerPriority`] that runs with late priority.
pub struct Late;

impl HandlerPriority for Late {
    fn priority(&self) -> i32 {
        i32::MIN / 2
    }
}

/// [`HandlerPriority`] that runs with last priority.
pub struct Last;

impl HandlerPriority for Last {
    fn priority(&self) -> i32 {
        i32::MIN
    }
}
