use std::fmt::{self, Debug, Formatter};

use bevy::ecs::system::{Command, EntityCommands};

use crate::prelude::*;

/// A state that an entity may be in. A state must implement [`Reflect`], but a workaround exists
/// for structs that contain types that do not implement [`Reflect`].
///
/// ```rust
/// # use bevy::prelude::*;
/// #
/// #[derive(Clone)]
/// struct NonReflectType;
///
/// #[derive(Clone, Component, Reflect)]
/// #[component(storage = "SparseSet")]
/// struct MyState {
///     #[reflect(ignore)]
///     non_reflect_type: NonReflectType
/// }
/// ```
///
/// This workaround currently does not affect the functionality of your state machine.
///
/// If you are concerned with performance, consider having your states use sparse set storage,
/// since they may be added to and removed from entities.
pub trait MachineState: Component + Clone + Reflect {}

impl<T: Component + Clone + Reflect> MachineState for T {}

/// State that represents any state. Transitions from [`AnyState`] may transition
/// from any other state.
#[derive(Clone, Component, Debug, Reflect)]
pub enum AnyState {}

#[derive(Debug)]
pub(crate) enum OnEvent {
    Entity(Box<dyn EntityEvent>),
    Command(Box<dyn CommandEvent>),
}

impl OnEvent {
    pub(crate) fn trigger(&self, entity: Entity, commands: &mut Commands) {
        match self {
            OnEvent::Entity(event) => event.trigger(&mut commands.entity(entity)),
            OnEvent::Command(event) => event.trigger(commands),
        }
    }
}

pub(crate) trait EntityEvent: Send + Sync {
    fn trigger(&self, entity: &mut EntityCommands);
}

impl Debug for dyn EntityEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Fn(&mut EntityCommands)")
    }
}

impl<F: Fn(&mut EntityCommands) + Send + Sync> EntityEvent for F {
    fn trigger(&self, entity: &mut EntityCommands) {
        self(entity)
    }
}

pub(crate) trait CommandEvent: Command + Sync {
    fn trigger(&self, commands: &mut Commands);
}

impl Debug for dyn CommandEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Command")
    }
}

impl<C: Clone + Command + Sync> CommandEvent for C {
    fn trigger(&self, commands: &mut Commands) {
        commands.add(self.clone())
    }
}

// An attempt to rebuild the state bundle from the world:

// struct StateMarker<T: MachineState>(PhantomData<T>);
//
// impl<T: MachineState> StateMarker<T> {
//     fn get(world: &World, entity: Entity, state: Box<dyn DynState>) -> &T {
//         let bundles = world.bundles();
//         let components = bundles
//             .get(bundles.get_id(TypeId::of::<T>()).unwrap())
//             .unwrap()
//             .components()
//             .iter()
//             .map(|component| {
//                 (
//                     world
//                         .components()
//                         .get_info(*component)
//                         .unwrap()
//                         .type_id()
//                         .unwrap(),
//                     world.get_by_id(entity, *component).unwrap(),
//                 )
//             })
//             .collect::<HashMap<_, _>>();
//
//         if let Some(component) = components.get(&state.type_id()) {
//             return unsafe { component.deref() }
//         }
//
//         match state.get_type_info() {
//             TypeInfo::Struct(info) => {
//                 let val = DynamicStruct::default();
//                 for field in info.iter() {
//                     let component = components.get(&field.type_id()).unwrap();
//                     val.insert(field.name(), unsafe { component.deref() }.);
//
//                 },
//             }
//         }
//     }
// }
