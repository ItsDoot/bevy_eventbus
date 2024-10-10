mod app;
mod config;
mod event;
mod input;
mod registry;
mod system;
mod world;

pub use app::*;
pub use config::*;
pub use event::*;
pub use input::*;
pub use registry::*;
pub use system::*;
pub use world::*;

#[cfg(test)]
mod tests {
    use bevy_ecs::{
        entity::Entity,
        system::{Commands, ResMut, Resource},
        world::World,
    };

    use crate::{
        CommandEventBus, Early, Event, First, Immutable, IntoHandlerConfig, Last, Mutable, Receive,
        WorldEventBus,
    };

    #[derive(Resource, Default)]
    struct Counter(i32);

    impl Counter {
        fn assert_order(&mut self, expected: i32) {
            assert_eq!(self.0, expected);
            self.0 += 1;
        }
    }

    struct Foo;

    impl Event for Foo {
        type Cancellation = bool;
        type Audience = Entity;
        type Mutability = Mutable;
    }

    struct Bar;

    impl Event for Bar {
        type Cancellation = bool;
        type Audience = ();
        type Mutability = Mutable;
    }

    struct Baz;

    impl Event for Baz {
        type Cancellation = bool;
        type Audience = ();
        type Mutability = Immutable;
    }

    #[test]
    fn event_cancellation_simple() {
        fn system(mut event: Receive<Bar>) {
            event.cancel();
        }

        let mut world = World::new();
        world.add_handler(system);

        let cancelled = world.post(Bar);
        assert!(cancelled);
    }

    #[test]
    fn event_cancellation_multistep() {
        fn step1(event: Receive<Bar>) {
            assert!(!event.cancelled());
        }

        fn step2(mut event: Receive<Bar>) {
            assert!(!event.cancelled());
            event.cancel();
            assert!(event.cancelled());
        }

        fn step3(_event: Receive<Bar>) {
            unreachable!();
        }

        let mut world = World::new();
        world.add_handler(step1);
        world.add_handler(step2);
        world.add_handler(step3);

        let cancelled = world.post(Bar);
        assert!(cancelled);
    }

    #[test]
    fn event_priority() {
        fn system1(_event: Receive<Bar>, mut counter: ResMut<Counter>) {
            counter.assert_order(0);
        }

        fn system2(_event: Receive<Bar>, mut counter: ResMut<Counter>) {
            counter.assert_order(1);
        }

        fn system3(_event: Receive<Bar>, mut counter: ResMut<Counter>) {
            counter.assert_order(2);
        }

        let mut world = World::new();
        world.init_resource::<Counter>();
        world.add_handler(system2.priority(Early));
        world.add_handler(system3.priority(Last));
        world.add_handler(system1.priority(First));

        world.post(Bar);
    }

    #[test]
    fn event_ordering() {
        fn system1(_event: Receive<Bar>, mut commands: Commands, mut counter: ResMut<Counter>) {
            counter.assert_order(0);
            commands.post(Baz);
        }

        fn system2(_event: Receive<Baz>, mut counter: ResMut<Counter>) {
            counter.assert_order(1);
        }

        fn system3(_event: Receive<Bar>, mut counter: ResMut<Counter>) {
            counter.assert_order(2);
        }

        let mut world = World::new();
        world.init_resource::<Counter>();
        world.add_handler(system1);
        world.add_handler(system2);
        world.add_handler(system3);

        world.post(Bar);
    }

    #[test]
    fn normal_system() {
        fn system(mut commands: Commands) {
            commands.post(Bar);
        }

        let mut world = World::new();
        world.add_handler(system);
    }
}
