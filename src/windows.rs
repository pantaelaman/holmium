use bevy::prelude::*;

pub fn create_window(mut commands: Commands) -> Entity {
  commands.spawn(NodeBundle { ..default() }).id()
}
