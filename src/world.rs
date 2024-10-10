use bevy_ecs::{
    system::Commands,
    world::{Command, World},
};

use crate::{
    Cancellation, Event, HandlerConfig, HandlerRegistry, Immutable, IntoHandlerConfig, Mutability,
    Mutable, Receive,
};

/// [`World`] extension trait for registering event handlers and posting events.
pub trait WorldEventBus {
    /// Adds an event handler for [`Event`] `E` to the world.
    fn add_handler<E: Event, M>(&mut self, system: impl IntoHandlerConfig<E, M>);

    /// Posts an [`Event`] to the world.
    fn post<E: Event<Audience = ()>>(&mut self, event: E) -> E::Cancellation {
        self.post_to(event, ())
    }

    /// Posts an [`Event`] to the world with a specific [`Audience`](Event::Audience).
    fn post_to<E: Event>(&mut self, event: E, audience: E::Audience) -> E::Cancellation;

    /// Posts an immutable reference to an [`Event`] to the world.
    fn post_ref<E: Event<Audience = (), Mutability = Immutable>>(
        &mut self,
        event: &E,
    ) -> E::Cancellation {
        self.post_ref_to(event, ())
    }

    /// Posts an immutable reference to an [`Event`] to the world with a specific [`Audience`](Event::Audience).
    fn post_ref_to<E: Event<Mutability = Immutable>>(
        &mut self,
        event: &E,
        audience: E::Audience,
    ) -> E::Cancellation;

    /// Posts a mutable reference to an [`Event`] to the world.
    fn post_mut<E: Event<Audience = (), Mutability = Mutable>>(
        &mut self,
        event: &mut E,
    ) -> E::Cancellation {
        self.post_mut_to(event, ())
    }

    /// Posts a mutable reference to an [`Event`] to the world with a specific [`Audience`](Event::Audience).
    fn post_mut_to<E: Event<Mutability = Mutable>>(
        &mut self,
        event: &mut E,
        audience: E::Audience,
    ) -> E::Cancellation;
}

impl WorldEventBus for World {
    fn add_handler<E: Event, M>(&mut self, handler: impl IntoHandlerConfig<E, M>) {
        let config = handler.into_config();
        config.handler.lock_arc().initialize(self);

        let mut registry = self.get_resource_or_insert_with(HandlerRegistry::<E>::default);
        registry.insert(config);
    }

    fn post_to<E: Event>(&mut self, mut event: E, audience: E::Audience) -> E::Cancellation {
        let Some(registry) = self.get_resource::<HandlerRegistry<E>>() else {
            return E::Cancellation::default();
        };

        let mut cancellation = E::Cancellation::default();

        let handlers = registry.handlers().cloned().collect::<Vec<_>>();
        for handler in handlers {
            let input = Receive::new(
                E::Mutability::to_ref(&mut event),
                cancellation.as_mut(),
                &audience,
            );
            handler.lock().run(input, self);

            if cancellation.cancelled() {
                break;
            }
        }

        cancellation
    }

    fn post_ref_to<E: Event<Mutability = Immutable>>(
        &mut self,
        event: &E,
        audience: E::Audience,
    ) -> E::Cancellation {
        let Some(registry) = self.get_resource::<HandlerRegistry<E>>() else {
            return E::Cancellation::default();
        };

        let mut cancellation = E::Cancellation::default();

        let handlers = registry.handlers().cloned().collect::<Vec<_>>();
        for handler in handlers {
            let input = Receive::new(event, cancellation.as_mut(), &audience);
            handler.lock().run(input, self);

            if cancellation.cancelled() {
                break;
            }
        }

        cancellation
    }

    fn post_mut_to<E: Event<Mutability = Mutable>>(
        &mut self,
        event: &mut E,
        audience: E::Audience,
    ) -> E::Cancellation {
        let Some(registry) = self.get_resource::<HandlerRegistry<E>>() else {
            return E::Cancellation::default();
        };

        let mut cancellation = E::Cancellation::default();

        let handlers = registry.handlers().cloned().collect::<Vec<_>>();
        for handler in handlers {
            let input = Receive::new(&mut *event, cancellation.as_mut(), &audience);
            handler.lock().run(input, self);

            if cancellation.cancelled() {
                break;
            }
        }

        cancellation
    }
}

/// [`Commands`] extension trait for registering event handlers and posting events.
pub trait CommandEventBus {
    /// Queues a [`Command`] that adds an event handler for [`Event`] `E` to the world.
    fn add_handler<E: Event, M>(&mut self, system: impl IntoHandlerConfig<E, M>);

    /// Queues a [`Command`] that posts an [`Event`] to the world.
    fn post<E: Event<Audience = ()> + Send>(&mut self, event: E) {
        self.post_to(event, ());
    }

    /// Queues a [`Command`] that posts an [`Event`] to the world with a specific [`Audience`](Event::Audience).
    fn post_to<E: Event<Audience: Send> + Send>(&mut self, event: E, audience: E::Audience);
}

impl CommandEventBus for Commands<'_, '_> {
    fn add_handler<E: Event, M>(&mut self, system: impl IntoHandlerConfig<E, M>) {
        self.queue(AddHandler {
            system: system.into_config(),
        });
    }

    fn post_to<E: Event<Audience: Send> + Send>(&mut self, event: E, audience: E::Audience) {
        self.queue(PostEvent { event, audience });
    }
}

/// [`Command`] that adds a [`HandlerSystem`] to the [`World`].
///
/// [`HandlerSystem`]: crate::HandlerSystem
pub struct AddHandler<E: Event> {
    system: HandlerConfig<E>,
}

impl<E: Event> Command for AddHandler<E> {
    fn apply(self, world: &mut World) {
        world.add_handler(self.system);
    }
}

/// [`Command`] that posts an [`Event`] to the [`World`].
pub struct PostEvent<E: Event> {
    event: E,
    audience: E::Audience,
}

impl<E: Event<Audience: Send> + Send> Command for PostEvent<E> {
    fn apply(self, world: &mut World) {
        world.post_to(self.event, self.audience);
    }
}
