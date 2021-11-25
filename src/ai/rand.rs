use rand::seq::SliceRandom;

use crate::Game;

/// Randomly select a turn.
pub fn run<G: Game>(game: &G) -> G::Turn {
    game.turns()
        .choose(&mut rand::thread_rng())
        .unwrap()
        .clone()
}
