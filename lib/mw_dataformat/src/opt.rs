use mw_common::plid::PlayerId;

use crate::msg::Msg;

#[derive(Default)]
pub struct OptimizerState {
    // TODO
}

/// Output from an optimization pass
pub enum OptimizerResult {
    /// Further runs will not optimize things further; proceed to encoding.
    Good,
    /// Further runs might optimize things further; run again if you want to try.
    Dirty,
    /// The data is bad/invalid, the input sequence is malformed. Data is now clobbered.
    Bad,
}

/// Performs a **lossy** optimization pass on the given sequence of update messages.
///
/// These lossy optimizations rely on knowledge of the game mechanics
/// and the implementation details of the game!
///
/// Of particular note: the sorting order of the `Msg` enum.
///
/// Only valid for the standard MineWars game mode. For custom game modes, do not use!
///
/// This is very ugly and flakey, but it saves a lot on message size. We want the smallest streams and replay files.
///
/// If you notice any weird bugs in the game, try disabling this optimizer, to know if the issue is caused by it.
/// They might be caused by bugs in the optimizations.
pub fn optimize_mwseq_lossy(_state: &mut OptimizerState, _me: PlayerId, sequence: &mut Vec<Msg>) -> OptimizerResult {
    let r = OptimizerResult::Good;

    if sequence.is_empty() {
        return r;
    }

    // First, sort. This will expose all further optimizations.
    sequence.sort_unstable();

    // Remove any Nops at the end (Nops are sorted last for this reason)
    truncate_nops(sequence);
    if sequence.is_empty() {
        return r;
    }

    // Remove any literal duplicates.
    sequence.dedup();

    // TODO: implement more optimization passes

    r
}

fn truncate_nops(sorted_sequence: &mut Vec<Msg>) {
    let newlen = sorted_sequence.iter().rposition(|msg| *msg != Msg::Nop).unwrap_or(0);
    sorted_sequence.truncate(newlen);
}
