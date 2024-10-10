use std::{any::TypeId, sync::Arc};

use bevy_ecs::{
    archetype::ArchetypeComponentId,
    component::ComponentId,
    prelude::*,
    query::Access,
    schedule::InternedSystemSet,
    world::{unsafe_world_cell::UnsafeWorldCell, DeferredWorld},
};
use parking_lot::Mutex;

use crate::{Event, HandlerConfig, Immutable, IntoHandlerConfig, Receive};

/// An [`Event`] that represents a tick of the app update loop.
pub struct Tick;

impl Event for Tick {
    type Cancellation = ();
    type Audience = ();
    type Mutability = Immutable;
}

#[doc(hidden)]
pub struct TickSystemMarker;

impl<Marker, S: IntoSystem<(), (), Marker>> IntoHandlerConfig<Tick, (TickSystemMarker, Marker)>
    for S
{
    fn into_config(self) -> HandlerConfig<Tick> {
        let system = Arc::new(Mutex::new(TickSystem(IntoSystem::into_system(self))));
        HandlerConfig::new(system)
    }
}

pub(crate) struct TickSystem<S: System<In = (), Out = ()>>(S);

impl<S: System<In = (), Out = ()>> System for TickSystem<S> {
    type In = Receive<'static, Tick>;
    type Out = ();

    fn name(&self) -> std::borrow::Cow<'static, str> {
        self.0.name()
    }

    fn type_id(&self) -> TypeId {
        self.0.type_id()
    }

    fn component_access(&self) -> &Access<ComponentId> {
        self.0.component_access()
    }

    fn archetype_component_access(&self) -> &Access<ArchetypeComponentId> {
        self.0.archetype_component_access()
    }

    fn is_send(&self) -> bool {
        self.0.is_send()
    }

    fn is_exclusive(&self) -> bool {
        self.0.is_exclusive()
    }

    fn has_deferred(&self) -> bool {
        self.0.has_deferred()
    }

    unsafe fn run_unsafe(
        &mut self,
        _input: SystemIn<'_, Self>,
        world: UnsafeWorldCell,
    ) -> Self::Out {
        self.0.run_unsafe((), world)
    }

    fn run(&mut self, _input: SystemIn<'_, Self>, world: &mut World) -> Self::Out {
        self.0.run((), world)
    }

    fn apply_deferred(&mut self, world: &mut World) {
        self.0.apply_deferred(world)
    }

    fn queue_deferred(&mut self, world: DeferredWorld) {
        self.0.queue_deferred(world)
    }

    unsafe fn validate_param_unsafe(&self, world: UnsafeWorldCell) -> bool {
        self.0.validate_param_unsafe(world)
    }

    fn validate_param(&mut self, world: &World) -> bool {
        self.0.validate_param(world)
    }

    fn initialize(&mut self, world: &mut World) {
        self.0.initialize(world)
    }

    fn update_archetype_component_access(&mut self, world: UnsafeWorldCell) {
        self.0.update_archetype_component_access(world)
    }

    fn check_change_tick(&mut self, change_tick: bevy_ecs::component::Tick) {
        self.0.check_change_tick(change_tick)
    }

    fn default_system_sets(&self) -> Vec<InternedSystemSet> {
        self.0.default_system_sets()
    }

    fn get_last_run(&self) -> bevy_ecs::component::Tick {
        self.0.get_last_run()
    }

    fn set_last_run(&mut self, last_run: bevy_ecs::component::Tick) {
        self.0.set_last_run(last_run)
    }
}
