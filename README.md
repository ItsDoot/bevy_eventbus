# bevy_eventbus

An experiment with an alternative event system in [Bevy Engine](https://bevyengine.org/).

## W

## Why not use [observers](https://docs.rs/bevy/latest/bevy/ecs/prelude/struct.Observer.html)?

Observers:
- Don't have a specified ordering (outside of event bubbling).
- Don't support cancellation.
- Don't support unmodifiable event types.
- Don't support processing multiple targeted entities at the same time.

## Prior Art

- [evenio](https://crates.io/crates/evenio)
- [SpongePowered](https://docs.spongepowered.org/stable/en/plugin/event/listeners.html)