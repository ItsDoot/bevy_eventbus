use std::{
    borrow::{Borrow, BorrowMut},
    ops::{Deref, DerefMut},
};

use bevy_ecs::{entity::Entity, system::SystemInput};

use crate::{
    Cancellable, CancellableWith, Cancellation, CancellationMut, Event, MutabilityRef, Mutable,
    Unicast,
};

/// [`SystemInput`] type for receiving events in handlers.
pub struct Receive<'event, E: Event> {
    /// The event being received.
    event: MutabilityRef<'event, E>,
    /// The cancellation state of the event.
    cancellation: CancellationMut<'event, E>,
    /// The intended audience of the event.
    audience: &'event E::Audience,
}

impl<'event, E: Event> Receive<'event, E> {
    /// Creates a new [`Receive`] instance.
    pub fn new(
        event: MutabilityRef<'event, E>,
        cancellation: CancellationMut<'event, E>,
        audience: &'event E::Audience,
    ) -> Self {
        Self {
            event,
            cancellation,
            audience,
        }
    }

    /// Returns a read-only reference to the event.
    pub fn event(&self) -> &E {
        self.event.borrow()
    }

    /// Returns a mutable reference to the event.
    /// Requires the [`Event`] `E` to be [`Mutable`].
    pub fn event_mut(&mut self) -> &mut E
    where
        E: Event<Mutability = Mutable>,
    {
        self.event
    }

    /// Returns `true` if the event was cancelled.
    /// This will always return `false` if the [`Event`] `E` is not
    /// [`Cancellable`] or [`CancellableWith`].
    ///
    /// To cancel an event, use [`Receive::cancel`].
    pub fn cancelled(&self) -> bool {
        self.cancellation.borrow().cancelled()
    }

    /// Cancels the event from being processed further.
    /// Requires the [`Event`] `E` to be [`Cancellable`].
    ///
    /// To check if an event is cancelled, use [`Receive::cancelled`].
    pub fn cancel(&mut self)
    where
        E: Event<Cancellation: Cancellable>,
    {
        self.cancellation.borrow_mut().cancel();
    }

    /// Cancels the event from being processed further with a value.
    /// Requires the [`Event`] `E` to be [`CancellableWith`] `T`.
    ///
    /// To check if an event is cancelled, use [`Receive::cancelled`].
    pub fn cancel_with<T>(&mut self, value: T)
    where
        E: Event<Cancellation: CancellableWith<T>>,
    {
        self.cancellation.borrow_mut().cancel_with(value);
    }

    /// Returns the target entity of the event.
    pub fn target(&self) -> Entity
    where
        E: Event<Audience: Unicast>,
    {
        self.audience.target()
    }
}

impl<E: Event> SystemInput for Receive<'_, E> {
    type Param<'i> = Receive<'i, E>;
    type Inner<'i> = Receive<'i, E>;

    fn wrap(this: Self::Inner<'_>) -> Self::Param<'_> {
        this
    }
}

impl<E: Event> Deref for Receive<'_, E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        self.event.borrow()
    }
}

impl<E: Event<Mutability = Mutable>> DerefMut for Receive<'_, E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.event
    }
}
