use bevy::{
  animation::{
    AnimationClip, AnimationPlayer, EntityPath, Interpolation, VariableCurve,
  },
  asset::Assets,
  core::Name,
  ecs::{
    bundle::Bundle,
    component::Component,
    entity::Entity,
    event::EventReader,
    query::With,
    schedule::{NextState, State},
    system::{Commands, Query, Res, ResMut, Resource},
  },
  input::{keyboard::KeyCode, ButtonInput},
  log::info,
  math::Vec3,
  sprite::{SpriteSheetBundle, TextureAtlas},
};
use bevy_ecs_ldtk::{GridCoords, LdtkEntity};
use bevy_ecs_tilemap::tiles::TileStorage;
use std::ops::Deref;

use crate::{
  assets::AtlasInfo,
  tiles::{CURSOR, CURSOR_BLUE_OFFSET},
};

use super::{
  arrows::ArrowHead,
  input::MovementInput,
  units::{Unit, UnitAssociation, UnitAssociations},
  GameEntity, GameState, TurnState, UnitMap,
};

#[derive(Default, Component)]
pub struct Cursor;

#[derive(Default, Bundle, LdtkEntity)]
pub struct CursorBundle {
  cursor: Cursor,
  name: Name,
  game_entity: GameEntity,
  pub animation_player: AnimationPlayer,
  #[sprite_sheet_bundle]
  pub sprite_bundle: SpriteSheetBundle,
  #[grid_coords]
  pub grid_coords: GridCoords,
}

pub fn init_cursor(
  mut commands: Commands,
  atlas_info: Res<AtlasInfo>,
  mut animations: ResMut<Assets<AnimationClip>>,
) {
  info!("initialising cursor");
  let mut cursor_bundle = CursorBundle::default();
  cursor_bundle.sprite_bundle.texture = atlas_info.image.clone();
  cursor_bundle.sprite_bundle.atlas = TextureAtlas {
    layout: atlas_info.layout.clone(),
    index: CURSOR,
  };
  cursor_bundle.sprite_bundle.transform.translation.z = 25.0;
  let name = Name::new("cursor");

  let mut animation = AnimationClip::default();
  animation.add_curve_to_path(
    EntityPath {
      parts: vec![name.clone()],
    },
    VariableCurve {
      keyframe_timestamps: vec![0.0, 0.5, 1.0],
      keyframes: bevy::animation::Keyframes::Scale(vec![
        Vec3::new(1.0, 1.0, 1.0),
        Vec3::new(1.2, 1.2, 1.0),
        Vec3::new(1.0, 1.0, 1.0),
      ]),
      interpolation: Interpolation::Linear,
    },
  );

  let mut player = AnimationPlayer::default();
  player.play(animations.add(animation)).repeat();

  cursor_bundle.animation_player = player;
  cursor_bundle.name = name;

  commands.spawn(cursor_bundle);
}

#[derive(Default, Component)]
pub struct Targeted;

pub fn move_cursor(
  mut commands: Commands,
  mut cursor: Query<&mut GridCoords, With<Cursor>>,
  keys: Res<ButtonInput<KeyCode>>,
  mut movement_events: EventReader<MovementInput>,
  turn_state: Res<State<TurnState>>,
  unit_associations: Res<UnitAssociations>,
  mut next_game_state: ResMut<NextState<GameState>>,
  units: Query<(&Unit, &UnitAssociation)>,
  unit_storage: Query<&TileStorage, With<UnitMap>>,
) {
  for movement_event in movement_events.read() {
    *cursor.single_mut() += movement_event.as_grid_coords();
  }

  if keys.just_pressed(KeyCode::Enter) {
    if let Some(entity) = unit_storage
      .single()
      .checked_get(&crate::util::grid_to_tile(*cursor.single()))
    {
      let (unit, association) = units.get(entity).unwrap();
      if association.turn == *turn_state.get() && !unit.moved {
        next_game_state.set(GameState::ArrowMovement);
        commands.entity(entity).insert(Targeted);
        commands.insert_resource(ArrowHead(cursor.single().clone()));
        info!("targeting unit {:?}", entity);
      }
    }
  }
}
