use crate::{
  assets::{LdtkWorldHandle, ATLAS_INFO},
  GlobalState,
};
use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_ecs_ldtk::prelude::*;
use bevy_ecs_tilemap::{
  helpers::geometry::get_tilemap_center_transform,
  map::{
    TilemapGridSize, TilemapSize, TilemapTexture, TilemapTileSize, TilemapType,
  },
  tiles::TileStorage,
  TilemapBundle,
};

use self::input::MovementInput;

pub mod arrows;
pub mod cursor;
pub mod input;

#[derive(Default, Debug, PartialEq, Eq, Hash, Clone, Copy, States)]
pub enum TurnState {
  #[default]
  Player1,
  Player2,
}

#[derive(Default, Debug, PartialEq, Eq, Hash, Clone, Copy, States)]
pub enum GameState {
  #[default]
  CursorMovement,
  ArrowMovement,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
  fn build(&self, app: &mut App) {
    app
      .init_state::<TurnState>()
      .init_state::<GameState>()
      .add_event::<MovementInput>()
      .add_systems(
        OnEnter(GlobalState::Game),
        (init_world, cursor::init_cursor).chain(),
      )
      .add_systems(
        OnEnter(GameState::ArrowMovement),
        arrows::calculate_moveable_region,
      )
      .add_systems(OnExit(GameState::ArrowMovement), arrows::clear_drawn_arrows)
      .add_systems(
        Update,
        (
          crate::tiles::cache_tile_types,
          input::movement_events,
          cursor::move_cursor.run_if(in_state(GameState::CursorMovement)),
          arrows::move_arrow_head.run_if(in_state(GameState::ArrowMovement)),
          update_grid_coord_positions,
        )
          .chain()
          .run_if(in_state(GlobalState::Game)),
      );
  }
}

#[derive(Default, Component)]
struct GameEntity;

#[derive(Default, Component, Clone, Copy)]
pub struct ZoneMap;

#[derive(Default, Component, Clone, Copy)]
pub struct ArrowMap;

#[derive(Resource)]
pub struct GlobalCamera(Entity);

impl std::ops::Deref for GlobalCamera {
  type Target = Entity;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

#[derive(Resource)]
pub struct LevelSize {
  px_hei: i32,
  px_wid: i32,
  tile_hei: usize,
  tile_wid: usize,
}

impl LevelSize {
  fn as_tilemap_size(&self) -> TilemapSize {
    TilemapSize {
      x: self.tile_wid as u32,
      y: self.tile_hei as u32,
    }
  }
}

fn init_world(
  mut commands: Commands,
  ldtk_handle: Res<LdtkWorldHandle>,
  projects: Res<Assets<LdtkProject>>,
) {
  info!("Initialising game world");

  let level_selection = LevelSelection::index(0);

  let project = projects.get(ldtk_handle.clone()).unwrap();
  let level = project
    .data()
    .as_standalone()
    .find_loaded_level_by_level_selection(&level_selection)
    .unwrap();

  let level_size = LevelSize {
    px_hei: *level.px_hei(),
    px_wid: *level.px_wid(),
    tile_hei: *level.px_hei() as usize / 16,
    tile_wid: *level.px_wid() as usize / 16,
  };

  let mut camera = Camera2dBundle::default();
  camera.projection.scaling_mode = ScalingMode::AutoMin {
    min_width: *level.px_wid() as f32,
    min_height: *level.px_hei() as f32,
  };
  camera.transform.translation.x = *level.px_wid() as f32 / 2.0;
  camera.transform.translation.y = *level.px_hei() as f32 / 2.0;
  let camera_id = commands.spawn(camera).id();
  commands.insert_resource(GlobalCamera(camera_id));

  commands.insert_resource(level_selection);

  commands.spawn((
    LdtkWorldBundle {
      ldtk_handle: ldtk_handle.clone(),
      ..default()
    },
    GameEntity,
  ));

  commands.spawn(create_tilemap(5.0, &level_size, ZoneMap));
  commands.spawn(create_tilemap(20.0, &level_size, ArrowMap));

  commands.insert_resource(level_size);
}

fn create_tilemap<T: Bundle>(
  z: f32,
  level_size: &LevelSize,
  attachments: T,
) -> impl Bundle {
  let tile_storage = TileStorage::empty(TilemapSize {
    x: level_size.tile_wid as u32,
    y: level_size.tile_hei as u32,
  });

  let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
  let grid_size = tile_size.into();

  (
    TilemapBundle {
      grid_size,
      size: level_size.as_tilemap_size(),
      storage: tile_storage,
      map_type: TilemapType::Square,
      texture: TilemapTexture::Single(ATLAS_INFO.get().unwrap().image.clone()),
      tile_size,
      transform: Transform {
        translation: Vec3 {
          x: 8.0,
          y: 8.0,
          z,
          ..default()
        },
        ..default()
      },
      ..default()
    },
    attachments,
  )
}

fn update_grid_coord_positions(
  mut grid_coord_entities: Query<
    (&mut Transform, &GridCoords),
    (With<GameEntity>, Changed<GridCoords>),
  >,
) {
  for (mut transform, grid_coords) in grid_coord_entities.iter_mut() {
    transform.translation = bevy_ecs_ldtk::utils::grid_coords_to_translation(
      *grid_coords,
      IVec2::splat(16),
    )
    .extend(transform.translation.z);
  }
}
