use bevy::{
  ecs::{
    bundle::Bundle,
    component::Component,
    entity::Entity,
    event::EventReader,
    query::With,
    schedule::{NextState, State},
    system::{Commands, Query, Res, ResMut, Resource},
  },
  hierarchy::DespawnRecursiveExt,
  input::{keyboard::KeyCode, ButtonInput},
  sprite::{SpriteSheetBundle, TextureAtlas},
  utils::HashMap,
};
use bevy_ecs_ldtk::{GridCoords, LdtkEntity};
use bevy_ecs_tilemap::{
  map::TilemapId,
  tiles::{TileBundle, TilePos, TileStorage, TileTextureIndex},
};
use itertools::Itertools;

use crate::{
  assets::ATLAS_INFO,
  tiles::{
    TileTypes, ARROW_BODY_DL, ARROW_BODY_DR, ARROW_BODY_H, ARROW_BODY_UL,
    ARROW_BODY_UR, ARROW_BODY_V, ARROW_HEAD_D, ARROW_HEAD_L, ARROW_HEAD_R,
    ARROW_HEAD_U, ZONE_MOVE,
  },
};

use super::{
  cursor::Cursor, input::MovementInput, ArrowMap, GameEntity, GameState,
  TurnState, ZoneMap,
};

#[derive(Default, Component)]
pub struct ArrowChunk;

#[derive(Default, Bundle, LdtkEntity)]
pub struct ArrowChunkBundle {
  arrow_chunk: ArrowChunk,
  game_entity: GameEntity,
  #[sprite_sheet_bundle]
  pub sprite_bundle: SpriteSheetBundle,
  #[grid_coords]
  pub grid_coords: GridCoords,
}

#[derive(Default, Component)]
pub struct MovementZone;

#[derive(Bundle, LdtkEntity)]
pub struct MovementZoneBundle {
  movement_zone: MovementZone,
  game_entity: GameEntity,
  #[sprite_sheet_bundle]
  pub sprite_bundle: SpriteSheetBundle,
  #[grid_coords]
  pub grid_coords: GridCoords,
}

impl Default for MovementZoneBundle {
  fn default() -> Self {
    let mut sprite_bundle = SpriteSheetBundle::default();
    sprite_bundle.texture = ATLAS_INFO.get().unwrap().image.clone();
    sprite_bundle.atlas = TextureAtlas {
      layout: ATLAS_INFO.get().unwrap().layout.clone(),
      index: ZONE_MOVE,
    };
    sprite_bundle.transform.translation.z = 15.0;
    MovementZoneBundle {
      movement_zone: MovementZone,
      game_entity: GameEntity,
      sprite_bundle,
      grid_coords: GridCoords::default(),
    }
  }
}

#[derive(Resource)]
pub struct ArrowHead(pub GridCoords);

impl std::ops::Deref for ArrowHead {
  type Target = GridCoords;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl std::ops::DerefMut for ArrowHead {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

pub fn create_arrow_chunk(
  arrow_index: usize,
  grid_coords: GridCoords,
  tilemap_id: Entity,
) -> impl Bundle {
  (
    TileBundle {
      position: TilePos {
        x: grid_coords.x as u32,
        y: grid_coords.y as u32,
      },
      texture_index: TileTextureIndex(arrow_index as u32),
      tilemap_id: TilemapId(tilemap_id),
      ..Default::default()
    },
    ArrowChunk,
  )
}

#[inline]
pub fn arrow_index_turn_parity(
  arrow_index: usize,
  turn_state: &TurnState,
) -> usize {
  arrow_index + ((*turn_state == TurnState::Player2) as usize * 4)
}

pub fn move_arrow_head(
  mut commands: Commands,
  arrow_chunks: Query<Entity, With<ArrowChunk>>,
  arrow_map: Query<Entity, With<ArrowMap>>,
  mut arrow_head: ResMut<ArrowHead>,
  keys: Res<ButtonInput<KeyCode>>,
  moveable_region: Res<MoveableRegion>,
  current_turn_state: Res<State<TurnState>>,
  mut next_game_state: ResMut<NextState<GameState>>,
  mut movement_events: EventReader<MovementInput>,
) {
  if keys.just_pressed(KeyCode::Enter) {
    next_game_state.set(GameState::CursorMovement);
    return;
  }

  if movement_events.is_empty() {
    return;
  }

  let mut destination = **arrow_head;
  for movement_event in movement_events.read() {
    destination += movement_event.as_grid_coords();
  }

  if !(moveable_region.contains_key(&destination)
    || destination == **arrow_head)
  {
    return;
  }

  **arrow_head = destination;

  for arrow_chunk in arrow_chunks.iter() {
    commands.entity(arrow_chunk).despawn();
  }

  let mut current_coords = **arrow_head;

  if let Some(first_parent_coords) = moveable_region.get(&current_coords) {
    let distance = current_coords - *first_parent_coords;
    commands.spawn(create_arrow_chunk(
      arrow_index_turn_parity(
        match (distance.x, distance.y) {
          (-1, 0) => ARROW_HEAD_L,
          (1, 0) => ARROW_HEAD_R,
          (0, -1) => ARROW_HEAD_D,
          (0, 1) => ARROW_HEAD_U,
          _ => unreachable!(),
        },
        &current_turn_state,
      ),
      current_coords,
      arrow_map.single(),
    ));
  }

  while let Some((target_coords, next_coords)) = {
    moveable_region
      .get(&current_coords)
      .map(|target_coords| {
        moveable_region
          .get(target_coords)
          .map(|next_coords| (target_coords, next_coords))
      })
      .flatten()
  } {
    let current_distance = current_coords - *target_coords;
    let next_distance = *next_coords - *target_coords;

    let arrow_index = match (
      (current_distance.x, current_distance.y),
      (next_distance.x, next_distance.y),
    ) {
      ((-1, 0), (1, 0)) | ((1, 0), (-1, 0)) => ARROW_BODY_H,
      ((0, -1), (0, 1)) | ((0, 1), (0, -1)) => ARROW_BODY_V,
      ((-1, 0), (0, -1)) | ((0, -1), (-1, 0)) => ARROW_BODY_UR,
      ((-1, 0), (0, 1)) | ((0, 1), (-1, 0)) => ARROW_BODY_DR,
      ((1, 0), (0, -1)) | ((0, -1), (1, 0)) => ARROW_BODY_UL,
      ((1, 0), (0, 1)) | ((0, 1), (1, 0)) => ARROW_BODY_DL,
      _ => unreachable!(),
    };

    commands.spawn(create_arrow_chunk(
      arrow_index_turn_parity(arrow_index, &current_turn_state),
      *target_coords,
      arrow_map.single(),
    ));

    current_coords = *target_coords;
  }
}

pub fn clear_drawn_arrows(
  mut commands: Commands,
  arrow_chunks: Query<Entity, With<ArrowChunk>>,
  movement_zones: Query<(Entity, &TilePos), With<MovementZone>>,
  mut zone_map: Query<&mut TileStorage, With<ZoneMap>>,
) {
  for dead_entity in arrow_chunks.iter() {
    commands.entity(dead_entity).despawn();
  }

  for (movement_zone, position) in movement_zones.iter() {
    commands.entity(movement_zone).despawn_recursive();
    zone_map.single_mut().remove(position);
  }
}

#[derive(Resource)]
pub struct MoveableRegion(HashMap<GridCoords, GridCoords>);

impl std::ops::Deref for MoveableRegion {
  type Target = HashMap<GridCoords, GridCoords>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl std::ops::DerefMut for MoveableRegion {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

pub fn calculate_moveable_region(
  mut commands: Commands,
  cursor: Query<&GridCoords, With<Cursor>>,
  mut zone_map: Query<(Entity, &mut TileStorage), With<ZoneMap>>,
  tile_types: Res<TileTypes>,
) {
  let moveable = HashMap::from_iter(
    pathfinding::directed::dijkstra::dijkstra_reach(
      cursor.single(),
      |node, cost| node_neighbours_with_cost(&*tile_types, node, cost),
    )
    .filter_map(|item| item.parent.map(|parent| (item.node, parent))),
  );

  for moveable_coord in moveable.keys() {
    let tile_pos = TilePos {
      x: moveable_coord.x as u32,
      y: moveable_coord.y as u32,
    };
    let tile = commands
      .spawn((
        TileBundle {
          position: tile_pos,
          texture_index: TileTextureIndex(ZONE_MOVE as u32),
          tilemap_id: TilemapId(zone_map.single().0),
          ..Default::default()
        },
        MovementZone,
      ))
      .id();
    zone_map.single_mut().1.set(&tile_pos, tile);
  }

  commands.insert_resource(MoveableRegion(moveable));
}

fn node_neighbours_with_cost(
  tile_types: &TileTypes,
  node: &GridCoords,
  total_cost: usize,
) -> impl IntoIterator<Item = (GridCoords, usize)> {
  const MAXIMUM_COST: usize = 6;

  crate::util::neighbours(node)
    .into_iter()
    .filter_map(|node| {
      if tile_types.grassy.contains(&node) {
        Some((node, 1))
      } else if tile_types.forested.contains(&node) {
        Some((node, 2))
      } else {
        None
      }
    })
    .filter(|(_, cost)| total_cost + cost <= MAXIMUM_COST)
    .collect_vec()
}
