//! Gamesweet
//!
//! `gamesweet` is a library defining a common interface for board games.

use std::fmt::{Debug, Display};

use log::error;

pub mod ai;

pub trait Game: Clone + Debug + Display {
    type Player: Clone + Debug + Display + PartialEq;
    type Turn: Clone + Debug + Display;

    /// Get the current player.
    fn player(&self) -> Self::Player;

    /// Get all legal turns.
    fn turns(&self) -> Vec<Self::Turn>;

    /// Play a turn of the game.
    fn play(&mut self, turn: Self::Turn) -> bool;

    /// Check if the game is over.
    fn over(&self) -> bool;

    /// Get the winner of the game.
    fn winner(&self) -> Option<Self::Player>;

    /// Main loop for a game.
    fn main(mut self, config: Config<Self>) {
        while !self.over() {
            println!("{}", self);

            while !self.play(config.turn(&self)) {
                error!("could not play turn");
            }
        }

        println!("{}", self);
        match self.winner() {
            Some(player) => println!("Winner: {}", player),
            None => println!("It's a tie!"),
        }
    }
}

pub type TurnFn<G> = fn(&G) -> <G as Game>::Turn;

pub struct Config<G: Game> {
    player1: (G::Player, TurnFn<G>),
    player2: (G::Player, TurnFn<G>),
}

impl<G: Game> Config<G> {
    /// Create a new Config.
    pub fn new(player1: (G::Player, TurnFn<G>), player2: (G::Player, TurnFn<G>)) -> Config<G> {
        assert!(player1.0 != player2.0);
        Config { player1, player2 }
    }

    /// Get a turn for a player.
    pub fn turn(&self, game: &G) -> G::Turn {
        if self.player1.0 == game.player() {
            self.player1.1(game)
        } else if self.player2.0 == game.player() {
            self.player2.1(game)
        } else {
            panic!()
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
