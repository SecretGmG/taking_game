mod entry;
pub mod kayles;

use entry::Entry;
use std::collections::HashMap;
use std::hash::Hash;
use std::io::Write;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;
use std::{io, thread};

use crate::entry::{EntryData, ProcessingData};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct GameId(usize);

/// Provides the interface for evaluating an impartial game with the `Evaluator`.
pub trait Impartial: Sized {
    /// Returns the list of successor game states (i.e., possible moves).
    fn get_split_moves(&self) -> Vec<Vec<Self>>;

    /// Returns the maximum nimber this game could have, if known.
    fn get_max_nimber(&self) -> Option<usize> {
        None
    }
}

#[derive(Debug)]
struct Store<G>
where
    G: Impartial + Hash + Eq + Ord + Clone,
{
    by_game: HashMap<Arc<G>, GameId>,
    games: Vec<Arc<G>>,
    entries: Vec<Entry>,
}

impl<G> Store<G>
where
    G: Impartial + Hash + Eq + Ord + Clone,
{
    fn new() -> Self {
        Self {
            by_game: HashMap::new(),
            games: Vec::new(),
            entries: Vec::new(),
        }
    }

    fn intern(&mut self, game: G) -> GameId {
        if let Some(&id) = self.by_game.get(&game) {
            return id;
        }

        let id = GameId(self.games.len());
        let max_nimber = game.get_max_nimber();
        let game = Arc::new(game);
        self.by_game.insert(game.clone(), id);
        self.games.push(game);
        self.entries.push(Entry::new(max_nimber));
        id
    }
}

impl<G> Default for Evaluator<G>
where
    G: Impartial + Hash + Eq + Ord + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

/// Evaluates impartial games via memoized recursive computation of nimbers.
///
/// `G` is the game type, which must implement `Impartial<G>`.
#[derive(Debug, Clone)]
pub struct Evaluator<G>
where
    G: Impartial + Hash + Eq + Ord + Clone,
{
    store: Arc<Mutex<Store<G>>>,
    pub cancel_flag: Arc<AtomicBool>,
}

impl<G> Evaluator<G>
where
    G: Impartial + Hash + Eq + Ord + Clone,
{
    /// Constructs a new, empty evaluator.
    pub fn new() -> Evaluator<G> {
        Evaluator {
            store: Arc::new(Mutex::new(Store::new())),
            cancel_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Returns a list of all known positions with their computed nimbers.
    pub fn get_nimbers(&self) -> Vec<(G, usize)> {
        let store = self.store.lock().unwrap();
        store
            .games
            .iter()
            .zip(&store.entries)
            .filter_map(|(game, entry)| {
                let nimber = entry.get_nimber()?;
                Some(((**game).clone(), nimber))
            })
            .collect()
    }

    /// Returns the number of entries stored in the evaluator cache.
    pub fn get_cache_size(&self) -> usize {
        self.store.lock().unwrap().entries.len()
    }

    /// Returns the number of stubs, processing and done cache entries.
    pub fn get_cache_stats(&self) -> (usize, usize, usize) {
        let mut stub = 0;
        let mut processing = 0;
        let mut done = 0;
        let store = self.store.lock().unwrap();
        for entry in &store.entries {
            match &entry.data {
                EntryData::Stub { .. } => stub += 1,
                EntryData::Processing { .. } => processing += 1,
                EntryData::Done { .. } => done += 1,
            }
        }
        (stub, processing, done)
    }

    pub fn stop(&self) {
        self.cancel_flag.store(true, Ordering::Relaxed);
    }

    pub fn resume(&self) {
        self.cancel_flag.store(false, Ordering::Relaxed);
    }

    /// Computes the nimber of the given game.
    /// Returns `None` if cancelled mid-computation.
    pub fn get_nimber(&self, game: &G) -> Option<usize> {
        self.get_bounded_nimber(game, usize::MAX)
    }

    /// Computes the nimber of a game, but aborts early if it can be proven that the nimber exceeds the provided upper bound.
    pub fn get_bounded_nimber(&self, game: &G, bound: usize) -> Option<usize> {
        self.get_bounded_nimber_by_parts(std::slice::from_ref(game), bound)
    }

    /// Computes the nimber of the given game decomposed into its parts.
    /// Returns `None` if cancelled mid-computation.
    pub fn get_nimber_by_parts(&self, parts: &[G]) -> Option<usize> {
        self.get_bounded_nimber_by_parts(parts, usize::MAX)
    }

    /// Computes the nimber of a sum of game parts under a bound.
    ///
    /// The result is computed as the XOR of the nimbers of each part,
    /// stopping early if it becomes clear the nimber would exceed the bound.
    pub fn get_bounded_nimber_by_parts(&self, parts: &[G], bound: usize) -> Option<usize> {
        let part_ids = {
            let mut store = self.store.lock().unwrap();
            parts
                .iter()
                .cloned()
                .map(|part| store.intern(part))
                .collect::<Vec<_>>()
        };
        self.get_bounded_nimber_by_part_ids(&part_ids, bound)
    }

    fn get_bounded_nimber_by_part_ids(&self, parts: &[GameId], bound: usize) -> Option<usize> {
        if parts.is_empty() {
            return Some(0);
        }
        let mut modifier = 0;
        for &part in &parts[0..parts.len() - 1] {
            modifier ^= self.get_bounded_nimber_of_part(part, usize::MAX)?;
        }
        // The bound is adjusted with `| modifier` to ensure that the final XOR result
        // isn't incorrectly pruned: if any intermediate nimber exceeds the original bound,
        // but the XOR still stays within it, we don't want a false early exit.
        Some(modifier ^ self.get_bounded_nimber_of_part(*parts.last()?, bound | modifier)?)
    }

    /// Computes the nimber of a specific game part with an upper bound.
    /// Returns `None` if cancelled or if nimber exceeds the bound.
    fn get_bounded_nimber_of_part(&self, part: GameId, bound: usize) -> Option<usize> {
        if let Some(nimber) = self.store.lock().unwrap().entries[part.0].get_nimber() {
            return Some(nimber);
        }

        self.destub(part);

        loop {
            if self.cancel_flag.load(Ordering::Relaxed) {
                return None;
            }

            let nimber = {
                let store = self.store.lock().unwrap();
                store.entries[part.0]
                    .get_smallest_possible_nimber()
                    .unwrap()
            };

            if nimber > bound {
                return None;
            }

            if !self.try_rule_out_nimber(part, nimber)? {
                {
                    let mut store = self.store.lock().unwrap();
                    store.entries[part.0].data = EntryData::Done { nimber };
                }
                return Some(nimber);
            }
        }
    }

    /// Attempts to prove that the given `nimber` cannot be the nimber of `game`.
    /// Returns `Some(true)` if it was successfully ruled out,
    /// `Some(false)` if the `nimber` is actually valid,
    /// and `None` if cancelled before a conclusion.
    fn try_rule_out_nimber(&self, game: GameId, nimber: usize) -> Option<bool> {
        if let Some(max_nimber) = self.store.lock().unwrap().entries[game.0].max_nimber {
            if max_nimber < nimber {
                return Some(false);
            }
        }

        let mut still_unprocessed_moves = vec![];
        let mut ruled_out_nimber = false;

        loop {
            let parts_opt = {
                let mut store = self.store.lock().unwrap();
                store.entries[game.0].pop_unprocessed_move().unwrap()
            };

            let Some(parts) = parts_opt else { break };

            if self.cancel_flag.load(Ordering::Relaxed) {
                return None;
            }

            match self.get_bounded_nimber_by_part_ids(&parts, nimber) {
                Some(move_nimber) => {
                    {
                        let mut store = self.store.lock().unwrap();
                        store.entries[game.0].mark_impossible(move_nimber);
                    }
                    if nimber == move_nimber {
                        ruled_out_nimber = true;
                        break;
                    }
                }
                None => {
                    still_unprocessed_moves.push(parts);
                }
            }
        }

        {
            let mut store = self.store.lock().unwrap();
            store.entries[game.0].append_unprocessed_moves(still_unprocessed_moves);
        }

        Some(ruled_out_nimber)
    }

    /// Initializes the move list for a game that is still a stub.
    ///
    /// For each move, the resulting game parts are reduced by canceling out
    /// symmetric pairs (since they XOR to 0), then interned to compact `GameId`s.
    fn destub(&self, game: GameId) {
        let game_to_split = {
            let store = self.store.lock().unwrap();
            if !store.entries[game.0].is_stub() {
                return;
            }
            store.games[game.0].clone()
        };

        let mut moves = game_to_split.get_split_moves();
        let mut moves = {
            let mut store = self.store.lock().unwrap();
            moves
                .iter_mut()
                .map(|parts| {
                    remove_pairs(parts);
                    parts
                        .drain(..)
                        .map(|part| store.intern(part))
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        };

        moves.sort_unstable();
        moves.dedup();

        {
            let mut store = self.store.lock().unwrap();
            store.entries[game.0].data = EntryData::Processing {
                data: ProcessingData::new(moves),
            };
        }
    }
}

impl<G> Evaluator<G>
where
    G: Impartial + Hash + Eq + Clone + Ord + Send + Sync + 'static,
{
    pub fn print_nimber_and_stats_of_game(&self, game: G) -> Option<usize> {
        self.print_nimber_and_stats_of_games(vec![game])
    }

    pub fn print_nimber_and_stats_of_games(&self, games: Vec<G>) -> Option<usize> {
        let eval_for_worker = self.clone();
        let eval_for_monitor = self.clone();
        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_for_monitor = stop_flag.clone();
        let stop_for_worker = stop_flag.clone();

        let worker = thread::spawn(move || {
            let nimber = eval_for_worker.get_nimber_by_parts(&games);
            stop_for_worker.store(true, Ordering::Relaxed);
            nimber
        });

        let monitor = thread::spawn(move || {
            while !stop_for_monitor.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(100));
                let (stubs, processing, done) = eval_for_monitor.get_cache_stats();
                print!(
                    "\rstubs: {}, processing: {}, done: {}, total: {}",
                    stubs,
                    processing,
                    done,
                    stubs + processing + done
                );
                io::stdout().flush().unwrap();
            }
        });

        let nimber = worker.join().unwrap();
        monitor.join().unwrap();

        println!("\nNimber: {}", nimber.unwrap_or(0));
        nimber
    }
}

/// Removes consecutive pairs of equal elements in a sorted list.
/// Used to cancel out symmetric subgames when computing nimbers.
fn remove_pairs<G>(vec: &mut Vec<G>)
where
    G: Ord,
{
    vec.sort_unstable();

    let mut read = 0;
    let mut write = 0;

    while read + 1 < vec.len() {
        if vec[read] == vec[read + 1] {
            read += 2;
        } else {
            vec.swap(read, write);
            read += 1;
            write += 1;
        }
    }
    if read < vec.len() {
        vec.swap(read, write);
        write += 1;
    }
    vec.truncate(write);
}
