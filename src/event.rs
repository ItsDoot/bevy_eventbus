use std::{
    borrow::{Borrow, BorrowMut},
    fmt::Debug,
};

use bevy_ecs::entity::Entity;

pub mod tick;

/// Messages sent between event handlers.
///
/// # Configuration
///
/// [`Event`] types have 3 main parts of configuration.
///
/// ## [`Mutability`](Event::Mutability)
///
/// Determines if an event can be modified or not. This is useful for ensuring that events are not
/// modified when they shouldn't be.
///
/// The two possible implementations are [`Immutable`] and [`Mutable`].
///
/// ## [`Cancellation`](Event::Cancellation)
///
/// Determines if an event can be cancelled or not. This is useful for stopping an event from being
/// processed further.
///
/// The two provided implementations are [`bool`] and `()`, but custom implementations can be made.
///
/// ## [`Audience`](Event::Audience)
///
/// Determines who the event is intended for. This can be a single entity, multiple entities, or no
/// entities at all.
///
/// The provided implementations are `()`, [`Entity`], [`Vec<Entity>`], and `[Entity; N]`.
///
/// # Examples
///
/// ## Unmodifiable, uncancellable, no audience
///
/// ```rust
/// struct MyEvent(String);
///
/// impl Event for MyEvent {
///     type Mutability = Immutable;
///     type Cancellation = ();
///     type Audience = ();
/// }
///
/// fn my_handler_system(event: Receive<MyEvent>) {
///     println!("Received event: {}", event.event().0);
/// }
/// ```
///
/// ## Modifiable, cancellable, single entity audience
///
/// ```rust
/// struct MyEvent(i32);
///
/// impl Event for MyEvent {
///    type Mutability = Mutable;
///    type Cancellation = bool;
///    type Audience = Entity;
/// }
///
/// fn my_handler_system(event: Receive<MyEvent>) {
///     event.0 += 1;
///     if event.0 > 10 {
///         event.cancel();
///     }
///     println!("Received from: {}", event.target());
/// }
/// ```
pub trait Event: 'static {
    /// Whether the event can be modified.
    ///
    /// This is either [`Immutable`] or [`Mutable`].
    type Mutability: Mutability;
    /// The type of cancellation state for this event.
    type Cancellation: Cancellation;
    /// Who the event is intended for.
    type Audience: Audience;
}

/// [`Event`] configuration that determines if an event can be modified or not.
pub trait Mutability: sealed::Mutability {
    /// The type of reference to the data that this [`Mutability`] allows.
    ///
    /// This is either `&'event T` for [`Immutable`] or `&'event mut T` for [`Mutable`].
    type Ref<'event, T: ?Sized + 'event>: Borrow<T>;

    /// Converts a mutable reference into the allowed reference type.
    fn to_ref<T: ?Sized>(value: &mut T) -> Self::Ref<'_, T>;
}

/// [`Event`] [`Mutability`] that only allows read-only access.
pub struct Immutable;

impl Mutability for Immutable {
    type Ref<'event, T: ?Sized + 'event> = &'event T;

    fn to_ref<T: ?Sized>(value: &mut T) -> Self::Ref<'_, T> {
        value
    }
}

/// [`Event`] [`Mutability`] that allows read-write access.
pub struct Mutable;

impl Mutability for Mutable {
    type Ref<'event, T: ?Sized + 'event> = &'event mut T;

    fn to_ref<T: ?Sized>(value: &mut T) -> Self::Ref<'_, T> {
        value
    }
}

/// Shorthand for the type of reference that the [`Mutability`] allows for an [`Event`].
pub type MutabilityRef<'event, E> = <<E as Event>::Mutability as Mutability>::Ref<'event, E>;

/// [`Event`] cancellation state.
/// For the actual act of checking and cancelling an event,
/// see [`Cancellable`] and [`CancellableWith`].
///
/// Provided implementations:
/// - `()`: An uncancellable event.
/// - [`bool`]: A simple boolean flag.
/// - [`Option<T>`]: Cancellation with a reason.
pub trait Cancellation: Debug + Default {
    /// A mutable reference to the cancellation state.
    type Mut<'event>: BorrowMut<Self>
    where
        Self: 'event;

    /// Returns a mutable reference to the cancellation state.
    fn as_mut(&mut self) -> Self::Mut<'_>;

    /// Returns `true` if the event is cancelled.
    /// To cancel an event, use [`Cancellable::cancel`].
    fn cancelled(&self) -> bool;
}

/// [`Event`] configuration to allow them to be cancelled.
///
/// For checking the cancellation state, see [`Cancellation::cancelled`].
/// For cancelling an event with a value, see [`CancellableWith::cancel_with`].
pub trait Cancellable: Cancellation {
    /// Cancels the event from being processed further.
    /// To check if an event is cancelled, use [`Cancellation::cancelled`].
    fn cancel(&mut self);
}

/// [`Event`] configuration to allow them to be cancelled with a value.
///
/// For checking the cancellation state, see [`Cancellation::cancelled`].
/// For cancelling an event without a value, see [`Cancellable::cancel`].
pub trait CancellableWith<T>: Cancellation {
    /// Cancels the event from being processed further, with a value.
    /// To check if an event is cancelled, use [`Cancellation::cancelled`].
    fn cancel_with(&mut self, value: T);
}

impl Cancellation for bool {
    type Mut<'event> = &'event mut bool;

    fn as_mut(&mut self) -> Self::Mut<'_> {
        self
    }

    fn cancelled(&self) -> bool {
        *self
    }
}

impl Cancellable for bool {
    fn cancel(&mut self) {
        *self = true;
    }
}

impl CancellableWith<bool> for bool {
    fn cancel_with(&mut self, value: bool) {
        *self = value || self.cancelled();
    }
}

impl Cancellation for () {
    type Mut<'event> = ();

    fn as_mut(&mut self) -> Self::Mut<'_> {
        *self
    }

    fn cancelled(&self) -> bool {
        false
    }
}

impl<T: Debug + 'static> Cancellation for Option<T> {
    type Mut<'event> = &'event mut Option<T>;

    fn as_mut(&mut self) -> Self::Mut<'_> {
        self
    }

    fn cancelled(&self) -> bool {
        self.is_some()
    }
}

impl<T: Debug + Default + 'static> Cancellable for Option<T> {
    fn cancel(&mut self) {
        *self = Some(Default::default());
    }
}

impl<T: Debug + 'static> CancellableWith<T> for Option<T> {
    fn cancel_with(&mut self, value: T) {
        *self = Some(value);
    }
}

/// Shorthand for a mutable reference to the [`Cancellation`] state of an [`Event`].
pub type CancellationMut<'event, E> = <<E as Event>::Cancellation as Cancellation>::Mut<'event>;

/// Who the [`Event`] is intended for.
///
/// Provided implementations:
/// - `()`: No target entities.
/// - [`Entity`]: A single target entity.
pub trait Audience {}

impl Audience for () {}

/// [`Audience`] that denotes an [`Event`] is intended for multiple entities.
///
/// Provided implementations:
/// - [`Vec<Entity>`]: A collection of target entities.
/// - `[Entity; N]`: A fixed-size array of target entities.
pub trait Multicast: Audience {
    /// The target entities of the [`Event`].
    fn targets(&self) -> impl Iterator<Item = Entity> + '_;
}

impl Audience for Vec<Entity> {}

impl Multicast for Vec<Entity> {
    fn targets(&self) -> impl Iterator<Item = Entity> + '_ {
        self.iter().copied()
    }
}

impl<const N: usize> Audience for [Entity; N] {}

impl<const N: usize> Multicast for [Entity; N] {
    fn targets(&self) -> impl Iterator<Item = Entity> + '_ {
        self.iter().copied()
    }
}

/// [`Audience`] that denotes an [`Event`] is intended for a specific entity.
///
/// Provided implementations:
/// - [`Entity`]: A single target entity.
pub trait Unicast: Audience {
    /// The target entity of the [`Event`].
    fn target(&self) -> Entity;
}

impl Audience for Entity {}

impl Unicast for Entity {
    fn target(&self) -> Entity {
        *self
    }
}

impl Unicast for [Entity; 1] {
    fn target(&self) -> Entity {
        self[0]
    }
}

mod sealed {
    pub trait Mutability {}
    impl Mutability for super::Immutable {}
    impl Mutability for super::Mutable {}
}
