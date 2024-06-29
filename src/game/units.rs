use bevy::{
  ecs::{
    bundle::Bundle,
    component::Component,
    entity::Entity,
    event::EventReader,
    query::{Added, Changed, With},
    system::{Commands, EntityCommands, Query, ResMut, Resource},
    world::EntityWorldMut,
  },
  hierarchy::DespawnRecursiveExt,
  utils::HashSet,
};
use bevy_ecs_ldtk::{EntityInstance, GridCoords, LdtkEntity, LevelEvent};
use bevy_ecs_tilemap::{
  map::TilemapId,
  tiles::{TileBundle, TilePos, TileStorage, TileTextureIndex},
};

use crate::tiles::{
  BACKDROP_BLUE, BACKDROP_RED, UNIT_ARCHER, UNIT_BARBARIAN, UNIT_CLERIC,
  UNIT_KNIGHT, UNIT_WIZARD,
};

use super::{BackdropMap, GameEntity, TurnState, UnitMap};

#[derive(Component)]
pub struct Unit {
  pub health: i32,
  pub max_move_cost: usize,
  pub moved: bool,
  pub backdrop: Entity,
}

#[derive(Bundle)]
pub struct UnitBundle {
  unit: Unit,
  pub association: UnitAssociation,
  pub tile_bundle: TileBundle,
}

#[derive(Default, Component)]
pub struct Backdrop;

#[derive(Bundle)]
pub struct BackdropBundle {
  backdrop: Backdrop,
  pub tile_bundle: TileBundle,
}

impl BackdropBundle {
  pub fn new(
    turn_state: TurnState,
    grid_coords: GridCoords,
    backdrop_map: Entity,
  ) -> Self {
    BackdropBundle {
      backdrop: Backdrop,
      tile_bundle: TileBundle {
        position: TilePos {
          x: grid_coords.x as u32,
          y: grid_coords.y as u32,
        },
        texture_index: TileTextureIndex(match turn_state {
          TurnState::Player1 => BACKDROP_BLUE as u32,
          TurnState::Player2 => BACKDROP_RED as u32,
        }),
        tilemap_id: TilemapId(backdrop_map),
        ..Default::default()
      },
    }
  }
}

pub trait UnitType: Default + Bundle {
  fn get_texture_index() -> usize;
  fn get_max_move_cost() -> usize {
    6
  }

  fn new(
    grid_coords: GridCoords,
    association: TurnState,
    unit_map: Entity,
    backdrop: Entity,
  ) -> impl Bundle {
    create_unit::<Self>(grid_coords, association, unit_map, backdrop)
  }
}

#[derive(Default, Component)]
pub struct Wizard;
impl UnitType for Wizard {
  #[inline]
  fn get_texture_index() -> usize {
    UNIT_WIZARD
  }
}

#[derive(Default, Component)]
pub struct Archer;
impl UnitType for Archer {
  #[inline]
  fn get_texture_index() -> usize {
    UNIT_ARCHER
  }

  #[inline]
  fn get_max_move_cost() -> usize {
    10
  }
}

#[derive(Default, Component)]
pub struct Barbarian;
impl UnitType for Barbarian {
  #[inline]
  fn get_texture_index() -> usize {
    UNIT_BARBARIAN
  }
}

#[derive(Default, Component)]
pub struct Knight;
impl UnitType for Knight {
  #[inline]
  fn get_texture_index() -> usize {
    UNIT_KNIGHT
  }
}

#[derive(Default, Component)]
pub struct Cleric;
impl UnitType for Cleric {
  #[inline]
  fn get_texture_index() -> usize {
    UNIT_CLERIC
  }
}

pub fn create_unit<U: UnitType>(
  grid_coords: GridCoords,
  association: TurnState,
  unit_map: Entity,
  backdrop: Entity,
) -> impl Bundle {
  (
    UnitBundle {
      unit: Unit {
        health: 30,
        max_move_cost: U::get_max_move_cost(),
        moved: false,
        backdrop,
      },
      tile_bundle: TileBundle {
        position: TilePos {
          x: grid_coords.x as u32,
          y: grid_coords.y as u32,
        },
        texture_index: TileTextureIndex(U::get_texture_index() as u32),
        tilemap_id: TilemapId(unit_map),
        ..Default::default()
      },
      association: UnitAssociation { turn: association },
    },
    U::default(),
    GameEntity,
  )
}

#[derive(Default, Component)]
pub struct UnitAssociation {
  pub turn: TurnState,
}

#[derive(Default, Component)]
pub struct UnitSpawnLocation;

#[derive(Default, Bundle, LdtkEntity)]
pub struct UnitSpawnLocationBundle {
  unit_spawn_location: UnitSpawnLocation,
  #[with(association_from_ldtk_instance)]
  pub association: UnitAssociation,
  #[grid_coords]
  pub grid_coords: GridCoords,
}

fn association_from_ldtk_instance(
  instance: &EntityInstance,
) -> UnitAssociation {
  UnitAssociation {
    turn: match &instance.identifier[..1] {
      "B" => TurnState::Player1,
      "R" => TurnState::Player2,
      _ => unreachable!(),
    },
  }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UnitSpawnTypes {
  Wizard,
  Archer,
  Knight,
  Barbarian,
  Cleric,
}

impl UnitSpawnTypes {
  fn create_bundle<'c>(
    &self,
    commands: &'c mut Commands,
    grid_coords: GridCoords,
    association: TurnState,
    unit_map: Entity,
    backdrop_map: Entity,
  ) -> EntityCommands<'c> {
    let backdrop = commands
      .spawn(BackdropBundle::new(association, grid_coords, backdrop_map))
      .id();

    match self {
      Self::Wizard => commands.spawn(Wizard::new(
        grid_coords,
        association,
        unit_map,
        backdrop,
      )),
      Self::Archer => commands.spawn(Archer::new(
        grid_coords,
        association,
        unit_map,
        backdrop,
      )),
      Self::Knight => commands.spawn(Knight::new(
        grid_coords,
        association,
        unit_map,
        backdrop,
      )),
      Self::Barbarian => commands.spawn(Barbarian::new(
        grid_coords,
        association,
        unit_map,
        backdrop,
      )),
      Self::Cleric => commands.spawn(Cleric::new(
        grid_coords,
        association,
        unit_map,
        backdrop,
      )),
    }
  }
}

#[derive(Resource)]
pub struct UnitSpawnQueues {
  player1: Vec<UnitSpawnTypes>,
  player2: Vec<UnitSpawnTypes>,
}

impl Default for UnitSpawnQueues {
  fn default() -> Self {
    let default_queue = vec![
      UnitSpawnTypes::Knight,
      UnitSpawnTypes::Knight,
      UnitSpawnTypes::Barbarian,
      UnitSpawnTypes::Archer,
      UnitSpawnTypes::Archer,
    ];

    UnitSpawnQueues {
      player1: default_queue.clone(),
      player2: default_queue,
    }
  }
}

pub fn fill_unit_spawn_locations(
  mut commands: Commands,
  mut level_events: EventReader<LevelEvent>,
  mut unit_spawn_queues: ResMut<UnitSpawnQueues>,
  unit_spawn_locations: Query<
    (Entity, &UnitAssociation, &GridCoords),
    With<UnitSpawnLocation>,
  >,
  mut unit_map: Query<(Entity, &mut TileStorage), With<UnitMap>>,
  backdrop_map: Query<Entity, With<BackdropMap>>,
) {
  for level_event in level_events.read() {
    if let LevelEvent::Spawned(_) = level_event {
      for (unit_spawn_location, association, grid_coords) in
        unit_spawn_locations.iter()
      {
        let unit_tile = match association.turn {
          TurnState::Player1 => &mut unit_spawn_queues.player1,
          TurnState::Player2 => &mut unit_spawn_queues.player2,
        }
        .pop()
        .unwrap()
        .create_bundle(
          &mut commands,
          grid_coords.clone(),
          association.turn,
          unit_map.single().0,
          backdrop_map.single(),
        )
        .id();

        unit_map.single_mut().1.set(
          &TilePos {
            x: grid_coords.x as u32,
            y: grid_coords.y as u32,
          },
          unit_tile,
        );

        commands.entity(unit_spawn_location).despawn_recursive();
      }
    }
  }
}

pub fn update_backdrop_positions(
  mut commands: Commands,
  units: Query<(&Unit, &TilePos), Changed<TilePos>>,
) {
  for (unit, position) in units.iter() {
    let position = position.clone();
    commands.entity(unit.backdrop).add(
      move |mut entity_world: EntityWorldMut| {
        *entity_world.get_mut::<TilePos>().unwrap() = position;
      },
    );
  }
}

#[derive(Default, Resource)]
pub struct UnitAssociations {
  player1: HashSet<Entity>,
  player2: HashSet<Entity>,
}

impl UnitAssociations {
  pub fn contains(&self, entity: Entity, turn_state: TurnState) -> bool {
    match turn_state {
      TurnState::Player1 => &self.player1,
      TurnState::Player2 => &self.player2,
    }
    .contains(&entity)
  }

  fn insert(&mut self, entity: Entity, turn_state: TurnState) {
    match turn_state {
      TurnState::Player1 => &mut self.player1,
      TurnState::Player2 => &mut self.player2,
    }
    .insert(entity);
  }
}

pub fn update_unit_associations_resource(
  units: Query<(Entity, &UnitAssociation), Added<Unit>>,
  mut associations: ResMut<UnitAssociations>,
) {
  for (entity, association) in units.iter() {
    associations.insert(entity, association.turn);
  }
}

pub fn refresh_units(
  mut units: Query<&mut Unit>,
) {
  for mut unit in units.iter_mut() {
    unit.moved = true;
  }
}
