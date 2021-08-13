use bevy_ecs::{prelude::{Entity, World}, system::{Command, EntityCommands}};
use crate::prelude::error;

pub struct Children(pub Vec<Entity>);

pub fn despawn_with_children_recursive(world: &mut World, entity: Entity) {
    if let Some(mut children) = world.get_mut::<Children>(entity) {
        for e in std::mem::take(&mut children.0) {
            despawn_with_children_recursive(world, e);
        }
    }

    if !world.despawn(entity) {
        error!("Failed to despawn entity {:?}", entity);
    }
}

#[derive(Debug)]
pub struct DespawnRecursive {
    entity: Entity,
}

impl Command for DespawnRecursive {
    fn write(self: Box<Self>, world: &mut World) {
        despawn_with_children_recursive(world, self.entity);
    }
}

pub trait DespawnRecursiveExt {
    /// Despawns the provided entity and its children.
    fn despawn_recursive(&mut self);
}

impl<'a, 'b> DespawnRecursiveExt for EntityCommands<'a, 'b> {
    /// Despawns the provided entity and its children.
    fn despawn_recursive(&mut self) {
        let entity = self.id();
        self.commands().add(DespawnRecursive { entity });
    }
}