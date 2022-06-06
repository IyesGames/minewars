//! General helper algorithms

use crate::grid::{Coord, Pos};

/// What to do with each tile considered by the `flood` algorithm?
pub enum FloodSelect {
    /// Tile does not qualify
    No,
    /// Add tile to end of queue
    Yes,
    /// Add tile to start of queue (next tile to check)
    YesPrio,
}

/// Type of the queue data structure (re)used for `flood`
pub type FloodQ = std::collections::VecDeque<Pos>;

/// Floodfill algorithm
///
/// Initialize `q` with starting tiles to expand from.
///
/// `P` is predicate.
/// First arg is the tile currently being considered.
/// Second argument is source tile we came from.
/// Must maintain its own state to and not return
/// "yes" twice for the same tile.
pub fn flood<C: Coord, P>(
    q: &mut FloodQ,
    mut p: P,
)
where
    P: FnMut(C, C) -> FloodSelect,
{
    while let Some(orig) = q.pop_front() {
        let orig: C = orig.into();
        for c in orig.iter_n0() {
            match p(c, orig) {
                FloodSelect::YesPrio => {
                    q.push_front(c.into());
                }
                FloodSelect::Yes => {
                    q.push_back(c.into());
                }
                FloodSelect::No => {}
            }
        }
    }
}

/// Type of the queue data structure (re)used for `reach`
pub type ReachQ = Vec<(Pos, u16)>;

/// Reachability (pathfinding) algorithm
///
/// Implementation of greedy-best-first-search, because that algo explores the
/// fewest cells and will lead to the lowest memory usage. We only want to know
/// if a path exists, not what it is. No need to find an "optimal" path.
///
/// `P` is predicate. Return `Some(cost)`, where cost is distance to goal, or 0
/// if the given cell is the end goal. Return `None` if the given cell cannot be
/// stepped on.
///
/// Uses externally-provided queue to reuse memory allocations.
pub fn reach<C: Coord, P>(
    q: &mut ReachQ,
    start: C,
    p: P,
) -> bool
where
    P: Fn(C) -> Option<u16>,
{
    match p(start) {
        Some(0) => {
            return true;
        }
        Some(cost) => {
            q.clear();
            q.push((start.into(), cost));
        }
        None => {
            return false;
        }
    }

    let mut cur = 0;
    let mut best = 0;

    loop {
        let mut found = false;
        let cc: C = q[cur].0.into();
        for c in cc.iter_n0() {
            if let Some(cost) = p(c) {
                if cost == 0 {
                    found = true;
                } else if let Err(i) = q.binary_search_by(|c2| cost.cmp(&c2.1)) {
                    if i > best {
                        best = i;
                    } else {
                        best += 1;
                    }
                    if i <= cur {
                        cur += 1;
                    }
                    q.insert(i, (c.into(), cost));
                }
            }
        }

        if found {
            q.clear();
            return true;
        } else if best > cur {
            cur = best;
        } else if cur == 0 {
            q.clear();
            return false;
        } else {
            cur -= 1;
            best = cur;
        }
    }
}

