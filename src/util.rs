use bevy_ecs_ldtk::GridCoords;

pub fn neighbours(grid_coord: &GridCoords) -> impl IntoIterator<Item=GridCoords> {
  [
    GridCoords {
      x: grid_coord.x + 1,
      y: grid_coord.y,
    },
    GridCoords {
      x: grid_coord.x - 1,
      y: grid_coord.y,
    },
    GridCoords {
      x: grid_coord.x,
      y: grid_coord.y + 1,
    },
    GridCoords {
      x: grid_coord.x,
      y: grid_coord.y - 1,
    },
  ]
}

