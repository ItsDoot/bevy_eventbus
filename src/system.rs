use std::sync::Arc;

use bevy_ecs::system::{IntoSystem, System};
use parking_lot::Mutex;

use crate::{Event, Receive};

/// Trait for [`System`]s that handle [`Event`]s.
pub trait HandlerSystem<E: Event, Out = ()>: System<In = Receive<'static, E>, Out = Out> {}

impl<E: Event, Out, S: System<In = Receive<'static, E>, Out = Out>> HandlerSystem<E, Out> for S {}

/// Trait for types that can be converted into [`HandlerSystem`]s.
pub trait IntoHandlerSystem<E: Event, Out, Marker> {
    /// The type of [`HandlerSystem`] that this instance converts into.
    type System: HandlerSystem<E, Out>;

    /// Turns this value into its corresponding [`HandlerSystem`].
    fn into_system(self) -> Self::System;
}

/// Any [`System`] that [`Receive`]s an [`Event`] can be converted into a [`HandlerSystem`].
impl<
        E: Event,
        Out,
        Marker,
        S: IntoSystem<Receive<'static, E>, Out, Marker, System: HandlerSystem<E, Out>>,
    > IntoHandlerSystem<E, Out, Marker> for S
{
    type System = S::System;

    fn into_system(self) -> Self::System {
        IntoSystem::into_system(self)
    }
}

pub type ArcHandlerSystem<E, Out = ()> = Arc<Mutex<dyn HandlerSystem<E, Out>>>;
