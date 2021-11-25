use std::cmp::Ordering;
use std::ops::{Index, IndexMut};
use std::time::Instant;

use log::{debug, trace};
use rand::seq::SliceRandom;

use crate::Game;

const DURATION: u128 = 995;
const THRESHOLD: u32 = 3;
const EXPLORE: f64 = 1.414;

/// Run MCTS to select a turn.
pub fn run<G: Game>(game: &G) -> G::Turn {
    // Record time MCTS was started
    let now = Instant::now();

    // Create the game tree
    let game = game.clone();
    let mut tree = Tree::new(Box::new(game));
    tree.expand(tree.root); // expand at root

    // Return immediately if only one valid turn
    if tree[tree.root].children.len() == 1 {
        let root = &tree[tree.root];
        return tree[root.children[0]].action.clone().unwrap();
    }

    while now.elapsed().as_millis() < DURATION {
        // Select a leaf node to expand
        let mut leaf = tree.select();

        // Expand `leaf` if it's been simulated more than `THRESHOLD`
        if tree[leaf].sims > THRESHOLD {
            tree.expand(leaf);
            leaf = *tree[leaf]
                .children
                .choose(&mut rand::thread_rng())
                .unwrap_or(&leaf);
        }

        // Simulate at `leaf`
        let winner = tree[leaf].simulate();

        // Backpropagate the winner
        tree.backprop(leaf, winner);
    }

    // Find most simulated node
    let root = &tree[tree.root];
    debug!("idx: sims, wins%, priority");
    let best = &tree[*root
        .children
        .iter()
        .map(|idx| (idx, &tree[*idx]))
        .inspect(|(idx, node)| {
            debug!(
                "{:03}: {:4}, {:4.1}%, {:.6}",
                idx,
                node.sims,
                100. * (node.wins as f64) / (node.sims as f64),
                node.priority(tree[node.parent].sims),
            )
        })
        .max_by(|(_, a), (_, b)| a.sims.partial_cmp(&b.sims).unwrap_or(Ordering::Equal))
        .unwrap()
        .0];

    // Play most simulated node
    best.action.clone().unwrap()
}

/// The game tree from the current position.
#[derive(Debug)]
struct Tree<G: Game> {
    arena: Vec<Node<G>>,
    root: usize,
}

impl<G: Game> Tree<G> {
    /// Create a new Tree initialized with a root.
    fn new(state: Box<G>) -> Tree<G> {
        Tree {
            arena: vec![Node::new(0, usize::MAX, state, None)],
            root: 0,
        }
    }

    /// Explore the game tree.
    fn select(&self) -> usize {
        let mut node = &self[self.root]; // start at the root

        // Loop until `node` has no children
        while !node.children.is_empty() {
            // Get the child with the highest priority
            trace!("idx: priority");
            node = &self[*node
                .children
                .iter()
                .map(|idx| {
                    (
                        idx,
                        Node::priority(&self[*idx], self[self[*idx].parent].sims),
                    )
                })
                .inspect(|(idx, priority)| trace!("{:03}: {:.6}", idx, priority))
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(Ordering::Equal))
                .unwrap()
                .0];
            trace!("{:03} selected", node.idx);
        }

        node.idx
    }

    /// Expand a node to create children in the game tree.
    fn expand(&mut self, idx: usize) {
        // Iterate through actions to create children
        for action in self[idx].state.turns() {
            // Clone state and play action
            let mut state: G = *self[idx].state.clone();
            state.play(action.clone());

            // Add the new child
            self.arena.push(Node::new(
                self.arena.len(),
                idx,
                Box::new(state),
                Some(action),
            ));
            // Parent stores index of child
            let child = self.arena.last().unwrap().idx;
            self[idx].children.push(child);
        }
    }

    /// Backpropagate the result of a simulation.
    fn backprop(&mut self, mut idx: usize, winner: Option<G::Player>) {
        let winner = winner.unwrap_or_else(|| self[self.root].state.player());

        // Backpropagate until the root
        let null = self[self.root].parent;
        while idx != null {
            let node = &mut self[idx];

            // Update statistics of node
            // NOTE: The game state stores the next player, but in MCTS, each
            //       node represents the current player.
            if winner != node.state.player() {
                node.wins += 1;
            }
            node.sims += 1;

            // Ascend to parent
            idx = node.parent;
        }
    }
}

impl<G: Game> Index<usize> for Tree<G> {
    type Output = Node<G>;

    fn index(&self, idx: usize) -> &Self::Output {
        &self.arena[idx]
    }
}

impl<G: Game> IndexMut<usize> for Tree<G> {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        &mut self.arena[idx]
    }
}

/// A single state in the game tree.
#[derive(Debug)]
struct Node<G: Game> {
    // Position
    idx: usize,
    parent: usize,
    children: Vec<usize>,
    // State
    state: Box<G>,
    action: Option<G::Turn>,
    // Statistics
    wins: u32,
    sims: u32,
}

impl<G: Game> Node<G> {
    /// Create a new Node.
    fn new(idx: usize, parent: usize, state: Box<G>, action: Option<G::Turn>) -> Node<G> {
        Node {
            idx,
            parent,
            children: Vec::new(),
            state,
            action,
            wins: 0,
            sims: 0,
        }
    }

    /// Simulate the game from this node.
    fn simulate(&self) -> Option<G::Player> {
        // Create a copy of the current state to simulate
        let mut state = self.state.clone();

        while !state.over() {
            // Policy: select a random move
            let action = state
                .turns()
                .choose(&mut rand::thread_rng())
                .unwrap()
                .clone();
            state.play(action);
        }

        state.winner()
    }

    /// Calculate node priority
    fn priority(&self, psims: u32) -> f64 {
        // Extract UCB
        let wins = self.wins as f64;
        let sims = self.sims as f64;
        let psims = psims as f64;
        // Calculate UCB
        let exploit = wins / sims;
        let explore = EXPLORE * (psims.ln() / sims).sqrt();
        // Return priority
        match exploit + explore {
            x if x.is_finite() => x,
            _ => f64::INFINITY,
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
