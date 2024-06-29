use bevy::{
  app::{App, Plugin}, ecs::{
    bundle::Bundle,
    component::Component,
    event::EventReader,
    query::With,
    system::{Commands, Query, Resource},
  }, log::info, utils::HashSet
};
use bevy_ecs_ldtk::{
  app::LdtkIntCellAppExt, GridCoords, LdtkIntCell, LevelEvent,
};

pub const TILESET_WIDTH: usize = 30;
pub const TILESET_HEIGHT: usize = 11;

pub const EMPTY: usize = index(5, 6);

// grass variations
pub const GRASS: usize = index(0, 0);
pub const GRASS_TUFTS: usize = index(1, 0);
pub const GRASS_FLOWERS: usize = index(2, 0);

// water variations
pub const WATER_LAKE: usize = index(1, 2);
pub const WATER_LAKE_L: usize = index(0, 2);
pub const WATER_LAKE_R: usize = index(2, 2);
pub const WATER_LAKE_U: usize = index(1, 1);
pub const WATER_LAKE_D: usize = index(1, 3);
pub const WATER_LAKE_UL: usize = index(0, 1);
pub const WATER_LAKE_UR: usize = index(2, 1);
pub const WATER_LAKE_DL: usize = index(0, 3);
pub const WATER_LAKE_DR: usize = index(2, 3);
pub const WATER_LAKE_CORNER_UL: usize = index(2, 5);
pub const WATER_LAKE_CORNER_UR: usize = index(3, 5);
pub const WATER_LAKE_CORNER_DL: usize = index(1, 5);
pub const WATER_LAKE_CORNER_DR: usize = index(0, 5);
pub const WATER_RIVER_V: usize = index(3, 3);
pub const WATER_RIVER_H: usize = index(1, 4);
pub const WATER_RIVER_MOUTH_L: usize = index(0, 4);
pub const WATER_RIVER_MOUTH_R: usize = index(2, 4);
pub const WATER_RIVER_MOUTH_U: usize = index(3, 2);
pub const WATER_RIVER_MOUTH_D: usize = index(3, 4);
pub const WATER_RIVER_UL: usize = index(4, 0);
pub const WATER_RIVER_UR: usize = index(5, 0);
pub const WATER_RIVER_DL: usize = index(4, 1);
pub const WATER_RIVER_DR: usize = index(5, 1);

// arrow variations
pub const ARROW_BODY_V: usize = index(4, 3);
pub const ARROW_BODY_H: usize = index(6, 2);
pub const ARROW_BODY_UL: usize = index(5, 3);
pub const ARROW_BODY_UR: usize = index(6, 3);
pub const ARROW_BODY_DL: usize = index(5, 4);
pub const ARROW_BODY_DR: usize = index(6, 4);
pub const ARROW_HEAD_L: usize = index(5, 2);
pub const ARROW_HEAD_R: usize = index(7, 2);
pub const ARROW_HEAD_U: usize = index(4, 2);
pub const ARROW_HEAD_D: usize = index(4, 4);

// highlight/zone variations
pub const CURSOR: usize = index(7, 3);
pub const ZONE_MOVE: usize = index(7, 4);
pub const ZONE_MELEE: usize = index(19, 5);
pub const ZONE_RANGED: usize = index(18, 5);

// terrain variations
pub const FOREST_LIGHT: usize = index(4, 5);
pub const FOREST_HEAVY: usize = index(4, 6);
pub const MOUNTAINS: usize = index(5, 5);

// turn association backdrops
pub const BACKDROP_BLUE: usize = index(6, 5);
pub const BACKDROP_RED: usize = index(7, 5);

// units
pub const UNIT_WIZARD: usize = index(19, 7);
pub const UNIT_ARCHER: usize = index(20, 7);
pub const UNIT_BARBARIAN: usize = index(22, 7);
pub const UNIT_KNIGHT: usize = index(19, 8);
pub const UNIT_CLERIC: usize = index(23, 8);

#[inline]
const fn index(col: usize, row: usize) -> usize {
  row * TILESET_WIDTH + col
}

#[derive(Default, Component)]
pub struct Watery;
#[derive(Default, Bundle, LdtkIntCell)]
pub struct WateryBundle {
  watery: Watery,
}

#[derive(Default, Component)]
pub struct Grassy;
#[derive(Default, Bundle, LdtkIntCell)]
pub struct GrassyBundle {
  grassy: Grassy,
}

#[derive(Default, Component)]
pub struct Mountainous;
#[derive(Default, Bundle, LdtkIntCell)]
pub struct MountainousBundle {
  mountainous: Mountainous,
}

#[derive(Default, Component)]
pub struct Forested;
#[derive(Default, Bundle, LdtkIntCell)]
pub struct ForestedBundle {
  forested: Forested,
}

pub struct TilesPlugin;

impl Plugin for TilesPlugin {
  fn build(&self, app: &mut App) {
    app
      .register_ldtk_int_cell::<WateryBundle>(1)
      .register_ldtk_int_cell::<GrassyBundle>(2)
      .register_ldtk_int_cell::<MountainousBundle>(3)
      .register_ldtk_int_cell::<ForestedBundle>(4);
  }
}

#[derive(Debug, Resource)]
pub struct TileTypes {
  pub watery: HashSet<GridCoords>,
  pub grassy: HashSet<GridCoords>,
  pub mountainous: HashSet<GridCoords>,
  pub forested: HashSet<GridCoords>,
}

pub fn cache_tile_types(
  mut commands: Commands,
  mut level_events: EventReader<LevelEvent>,
  watery_tiles: Query<&GridCoords, With<Watery>>,
  grassy_tiles: Query<&GridCoords, With<Grassy>>,
  mountainous_tiles: Query<&GridCoords, With<Mountainous>>,
  forested_tiles: Query<&GridCoords, With<Forested>>,
) {
  for level_event in level_events.read() {
    if let LevelEvent::Spawned(_) = level_event {
      let tile_types = TileTypes {
        watery: HashSet::from_iter(watery_tiles.iter().copied()),
        grassy: HashSet::from_iter(grassy_tiles.iter().copied()),
        mountainous: HashSet::from_iter(mountainous_tiles.iter().copied()),
        forested: HashSet::from_iter(forested_tiles.iter().copied()),
      };

      commands.insert_resource(tile_types);
    }
  }
}
