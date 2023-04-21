use crate::api::ActionError;
use crate::math::{Tile, TileMath};
use crate::model::Farmland;
use crate::Game;

impl Game {
    pub fn ensure_tile_empty(&self, farmland: Farmland, tile: Tile) -> Result<(), ActionError> {
        if self.is_tile_empty(farmland, tile) {
            Ok(())
        } else {
            Err(ActionError::TileNotEmpty)
        }
    }

    pub fn is_tile_empty(&self, farmland: Farmland, tile: Tile) -> bool {
        if self
            .physics
            .get_body_at(farmland.space, tile.position())
            .is_ok()
        {
            return false;
        }

        if self
            .physics
            .get_barrier_at(farmland.space, tile.position())
            .is_some()
        {
            return false;
        }

        return true;
    }
}
