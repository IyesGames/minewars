use mw_common::driver::{Game, Host};
use mw_common::plid::*;

use crate::prelude::*;
use crate::map::Map;

pub struct SessionManager<G: Game> {
    sessions: Vec<Session<G>>,
}

pub struct Session<G: Game> {
    state: Arc<Mutex<SessionState<G>>>,
}

pub struct SessionState<G: Game> {
    alive_plids: Plids,
    map: Arc<Map>,
    game_init_data: Option<Box<G::InitData>>,
    game: G,
    host: TokioHost<G>,
}

struct TokioHost<G: Game> {
    msg_q: Vec<(Plids, G::OutEvent)>,
    scheds_cancel: CancellationToken,
    schedkinds_cancels: HashMap<G::SchedEvent, CancellationToken>,
}

impl<G: Game> Host<G> for TokioHost<G> {
    fn msg(&mut self, plids: Plids, event: G::OutEvent) {
        self.msg_q.push((plids, event));
    }
    fn sched(&mut self, time: Instant, event: G::SchedEvent) {
        let token_sched = if let Some(&token_kind) = self.schedkinds_cancels.get(&event) {
            token_kind.child_token()
        } else {
            let token_kind = self.scheds_cancel.child_token();
            let token_sched = token_kind.child_token();
            self.schedkinds_cancels.insert(event.clone(), token_kind);
            token_sched
        };
        tokio::spawn(sched::<G>(token_sched, session, time, event));
    }
    fn desched_all(&mut self, event: G::SchedEvent) {
        if let Some(token) = self.schedkinds_cancels.remove(&event) {
            token.cancel();
        }
    }
    fn game_over(&mut self) {
        self.scheds_cancel.cancel();
        // TODO ...
    }
}

async fn sched<G: Game>(
    cancel: CancellationToken,
    session: Arc<Mutex<Session<G>>>,
    time: Instant,
    event: G::SchedEvent,
) {
    tokio::select! {
        _ = cancel.cancelled() => {
            return;
        }
        _ = tokio::time::sleep_until(time) => {
            tokio::select! {
                _ = cancel.cancelled() => {
                    return;
                }
                session = session.lock() => {
                    session.game.unsched(&mut session.host, event);
                }
            }
        }
    }
}
