use bevy_ecs_ldtk::GridCoords;
use bevy_ecs_tilemap::tiles::TilePos;

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

pub fn grid_to_tile(grid_coords: GridCoords) -> TilePos {
  TilePos {
    x: grid_coords.x as u32,
    y: grid_coords.y as u32,
  }
}
