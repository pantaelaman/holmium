use bevy::prelude::*;
use bevy_ecs_ldtk::LdtkPlugin;
use bevy_ecs_tilemap::TilemapPlugin;
use bevy_embedded_assets::{EmbeddedAssetPlugin, PluginMode};
use bevy_titan::SpriteSheetLoaderPlugin;

mod assets;
mod game;
mod tiles;
mod util;
mod windows;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, States)]
pub enum GlobalState {
  #[default]
  Loading,
  Game,
}

fn main() {
  App::new()
    .add_plugins((
      EmbeddedAssetPlugin {
        mode: PluginMode::ReplaceDefault,
      },
      DefaultPlugins.set(ImagePlugin::default_nearest()),
      SpriteSheetLoaderPlugin,
      TilemapPlugin,
      LdtkPlugin,
    ))
    .insert_resource(Msaa::Off)
    .init_state::<GlobalState>()
    .add_plugins((
      assets::LoadAssetsPlugin,
      tiles::TilesPlugin,
      game::GamePlugin,
    ))
    .run();
}
