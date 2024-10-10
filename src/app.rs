use bevy_app::App;

use crate::{Event, HandlerRegistry, IntoHandlerConfig};

/// [`App`] extension trait for registering event handlers.
pub trait AppEventBus {
    /// Adds an event handler for [`Event`] `E` to the app.
    fn add_handler<E: Event, M>(&mut self, handler: impl IntoHandlerConfig<E, M>) -> &mut Self;
}

impl AppEventBus for App {
    fn add_handler<E: Event, M>(&mut self, handler: impl IntoHandlerConfig<E, M>) -> &mut Self {
        let config = handler.into_config();

        config.handler.lock_arc().initialize(self.world_mut());

        let mut registry = self
            .world_mut()
            .get_resource_or_insert_with(HandlerRegistry::<E>::default);
        registry.insert(config);

        self
    }
}
