use bevy::{
  ecs::{
    event::{Event, EventWriter},
    schedule::State,
    system::Res,
  },
  input::{keyboard::KeyCode, ButtonInput},
  math::Vec2,
};
use bevy_ecs_ldtk::GridCoords;

use super::TurnState;

#[derive(Event)]
pub struct MovementInput {
  up: bool,
  down: bool,
  left: bool,
  right: bool,
}

impl MovementInput {
  pub fn as_vec2(&self) -> Vec2 {
    Vec2 {
      x: (self.right as i32 - self.left as i32) as f32,
      y: (self.up as i32 - self.down as i32) as f32,
    }
  }

  pub fn as_grid_coords(&self) -> GridCoords {
    GridCoords {
      x: (self.right as i32 - self.left as i32),
      y: (self.up as i32 - self.down as i32),
    }
  }
}

pub fn movement_events(
  keys: Res<ButtonInput<KeyCode>>,
  turn_state: Res<State<TurnState>>,
  mut movement_events: EventWriter<MovementInput>,
) {
  match **turn_state {
    TurnState::Player1 => {
      if keys.any_just_pressed([
        KeyCode::KeyW,
        KeyCode::KeyA,
        KeyCode::KeyS,
        KeyCode::KeyD,
      ]) {
        movement_events.send(MovementInput {
          up: keys.just_pressed(KeyCode::KeyW),
          down: keys.just_pressed(KeyCode::KeyS),
          left: keys.just_pressed(KeyCode::KeyA),
          right: keys.just_pressed(KeyCode::KeyD),
        });
      }
    }
    TurnState::Player2 => {
      if keys.any_pressed([
        KeyCode::KeyI,
        KeyCode::KeyJ,
        KeyCode::KeyK,
        KeyCode::KeyL,
      ]) {
        movement_events.send(MovementInput {
          up: keys.just_pressed(KeyCode::KeyI),
          down: keys.just_pressed(KeyCode::KeyK),
          left: keys.just_pressed(KeyCode::KeyJ),
          right: keys.just_pressed(KeyCode::KeyL),
        });
      }
    }
  }
}
