use crate::{
  tiles::{TILESET_HEIGHT, TILESET_WIDTH},
  GlobalState,
};
use bevy::prelude::*;
use bevy_ecs_ldtk::assets::LdtkProject;
use std::sync::OnceLock;

pub static ATLAS_INFO: OnceLock<AtlasInfo> = OnceLock::new();

pub struct LoadAssetsPlugin;

impl Plugin for LoadAssetsPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Startup, load_assets).add_systems(
      Update,
      check_loading.run_if(in_state(GlobalState::Loading)),
    );
  }
}

#[derive(Resource, Clone)]
pub struct AtlasInfo {
  pub image: Handle<Image>,
  pub layout: Handle<TextureAtlasLayout>,
}

#[derive(Resource)]
pub struct LdtkWorldHandle(Handle<LdtkProject>);

impl std::ops::Deref for LdtkWorldHandle {
  type Target = Handle<LdtkProject>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

pub fn load_assets(
  mut commands: Commands,
  server: Res<AssetServer>,
  mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
  let atlas_info = AtlasInfo {
    image: server.load("tilemap.png"),
    layout: layouts.add(TextureAtlasLayout::from_grid(
      Vec2 { x: 16.0, y: 16.0 },
      TILESET_WIDTH,
      TILESET_HEIGHT,
      None,
      None,
    )),
  };
  let _ = ATLAS_INFO.set(atlas_info.clone());
  commands.insert_resource(atlas_info);
  commands.insert_resource(LdtkWorldHandle(server.load("holmium.ldtk")));
}

pub fn check_loading(
  server: Res<AssetServer>,
  atlas_info: Res<AtlasInfo>,
  mut next_global_state: ResMut<NextState<GlobalState>>,
) {
  if server.is_loaded_with_dependencies(&atlas_info.image) {
    next_global_state.set(GlobalState::Game);
  }
}
