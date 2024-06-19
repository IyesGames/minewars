use mw_common::driver::Game;

use crate::prelude::*;

/// Trait for abuse detection and prevention implementations.
///
/// Examples: rate limiters, server-side anti-cheat, etc.
///algorithms
/// Every time a user wants to interact with the server, the abuse detector will
/// be prompted about whether to allow the specific action the user wants to perform.
pub trait AbuseDetector<G: Game> {
    fn verify_game_input(&mut self, user: UserId, now: Instant, action: &G::InputAction) -> AbuseVerdict;
}

/// Unique identifier for a user.
///
/// TODO: Replace this with something integrated with whatever account
/// management solution we end up using.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserId(u64);

/// Decision made by an `AbuseDetector` regarding some action that a user wants to perform.
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum AbuseVerdict {
    /// User is okay, let them proceed to do what they want to do.
    Accept,
    /// User is suspect. Let them proceed to do what they want to do, but flag them.
    Flag {
        /// Score value to indicate how bad the violation is.
        severity: u32,
    },
    /// User is suspect. Do not allow them to do what they want to do.
    Deny {
        /// Score value to indicate how bad the violation is.
        severity: u32,
        /// Optionally, punish the user. If None, just flag them (like `Flag`).
        punishment: Option<AbusePunishment>,
    }
}

/// Punishment to be dealt to an abusive user.
#[derive(Clone, Eq, Hash)]
pub enum AbusePunishment {
    /// Show them a warning.
    Warn {
        /// The warning message, if any.
        /// This should probably be a localization key.
        message: Option<String>,
    },
    /// Kick them from the session.
    Kick {
        /// Do not allow them to reconnect for a duration of time.
        /// If None, they may reconnect immediately.
        timeout: Option<Duration>,
        /// The warning message, if any.
        /// This should probably be a localization key.
        message: Option<String>,
    },
    /// Ban their account (do not allow them to join to any gameplay session).
    Ban {
        /// Make the ban temporary. Allow them to play again after a duration of time.
        /// If None, the ban should be permanent.
        expiry: Option<Duration>,
        /// The warning message, if any.
        /// This should probably be a localization key.
        message: Option<String>,
    },
}

impl AbuseVerdict {
    pub fn severity(&self) -> u32 {
        match self {
            AbuseVerdict::Accept => 0,
            AbuseVerdict::Flag { severity } => *severity,
            AbuseVerdict::Deny { severity, .. } => *severity,
        }
    }
}

impl Ord for AbuseVerdict {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use AbuseVerdict::*;
        match (self, other) {
            (Accept, Accept) =>
                std::cmp::Ordering::Equal,
            (Flag { severity: a }, Flag { severity: b }) =>
                a.cmp(b),
            (Deny { severity: sa, punishment: pa }, Deny { severity: sb, punishment: pb }) =>
                match pa.cmp(pb) {
                    std::cmp::Ordering::Equal => sa.cmp(sb),
                    ord => ord,
                },
            (Accept, _) =>
                std::cmp::Ordering::Less,
            (_, Accept) =>
                std::cmp::Ordering::Greater,
            (Flag { .. }, Deny { .. }) =>
                std::cmp::Ordering::Less,
            (Deny { .. }, Flag { .. }) =>
                std::cmp::Ordering::Greater,
        }
    }
}

impl PartialOrd for AbuseVerdict {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for AbusePunishment {
    fn eq(&self, other: &Self) -> bool {
        use AbusePunishment::*;
        match (self, other) {
            (Warn { .. }, Warn { .. }) => true,
            (Kick { timeout: a, .. }, Kick { timeout: b, .. }) =>
                a == b,
            (Ban { expiry: a, .. }, Ban { expiry: b, .. }) =>
                a == b,
            _ => false,
        }
    }
}

impl Ord for AbusePunishment {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use AbusePunishment::*;
        match (self, other) {
            (Warn { .. }, Warn { .. }) =>
                std::cmp::Ordering::Equal,
            (Kick { timeout: None, .. }, Kick { timeout: None, .. }) =>
                std::cmp::Ordering::Equal,
            (Kick { timeout: None, .. }, Kick { timeout: Some(_), .. }) =>
                std::cmp::Ordering::Less,
            (Kick { timeout: Some(_), .. }, Kick { timeout: None, .. }) =>
                std::cmp::Ordering::Greater,
            (Kick { timeout: Some(a), .. }, Kick { timeout: Some(b), .. }) =>
                a.cmp(b),
            (Ban { expiry: None, .. }, Ban { expiry: None, .. }) =>
                std::cmp::Ordering::Equal,
            (Ban { expiry: None, .. }, Ban { expiry: Some(_), .. }) =>
                std::cmp::Ordering::Greater,
            (Ban { expiry: Some(_), .. }, Ban { expiry: None, .. }) =>
                std::cmp::Ordering::Less,
            (Ban { expiry: Some(a), .. }, Ban { expiry: Some(b), .. }) =>
                a.cmp(b),
            (Warn { .. }, Kick { .. }) =>
                std::cmp::Ordering::Less,
            (Warn { .. }, Ban { .. }) =>
                std::cmp::Ordering::Less,
            (Kick { .. }, Warn { .. }) =>
                std::cmp::Ordering::Greater,
            (Kick { .. }, Ban { .. }) =>
                std::cmp::Ordering::Less,
            (Ban { .. }, Warn { .. }) =>
                std::cmp::Ordering::Greater,
            (Ban { .. }, Kick { .. }) =>
                std::cmp::Ordering::Greater,
        }
    }
}

impl PartialOrd for AbusePunishment {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Combinator to use multiple abuse detectors.
///
/// Each of the detectors will be queried. All of them must accept.
///
/// The final returned verdict will be the most severe one out of all of them.
/// The severity score will be the sum of all severity scores.
///
/// This allows for easy modularity. For example, if you want to use both
/// a rate limiter and an anti-cheat (or multiple different anti-cheats).
pub struct AbuseMultiDetector<G: Game> {
    pub detectors: Vec<Box<dyn AbuseDetector<G>>>,
}

impl<G: Game> AbuseDetector<G> for AbuseMultiDetector<G> {
    fn verify_game_input(&mut self, user: UserId, now: Instant, action: &G::InputAction) -> AbuseVerdict {
        let (total_severity, final_verdict) = self.detectors.iter_mut()
            .map(|detector| detector.verify_game_input(user, now, action))
            .fold((0, AbuseVerdict::Accept), |(total, final_verdict), verdict| {
                (total + verdict.severity(), final_verdict.max(verdict))
            });
        match final_verdict {
            AbuseVerdict::Accept => AbuseVerdict::Accept,
            AbuseVerdict::Flag { severity } =>
                AbuseVerdict::Flag { severity: total_severity },
            AbuseVerdict::Deny { severity, punishment } =>
                AbuseVerdict::Deny { severity: total_severity, punishment },
        }
    }
}

/// Dummy abuse detector that always accepts everything
pub struct AlwaysAcceptAbuse;

impl<G: Game> AbuseDetector<G> for AlwaysAcceptAbuse {
    fn verify_game_input(&mut self, user: UserId, now: Instant, action: &G::InputAction) -> AbuseVerdict {
        AbuseVerdict::Accept
    }
}

/// Abuse detector for action-agnostic rate limiting.
///
/// Keeps track of when the given user last performed an action (any action)
/// and can be configured to issue verdicts if the duration of time is below
/// specific thresholds. You can add as many thresholds (and associated verdicts)
/// as you want. Will return the verdict associated with the lowest threshold
/// that the input duration is less than.
#[derive(Default, Clone)]
pub struct SimpleRateLimiter {
    thresholds: Vec<(Duration, AbuseVerdict)>,
    last_action_time: HashMap<UserId, Instant>,
}

impl SimpleRateLimiter {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn add_verdict(&mut self, threshold: Duration, verdict: AbuseVerdict) {
        let i = match self.thresholds.binary_search_by_key(&threshold, |(t, _)| *t) {
            Ok(i) => i,
            Err(i) => i,
        };
        self.thresholds.insert(i, (threshold, verdict));
    }
    pub fn with_verdict(mut self, threshold: Duration, verdict: AbuseVerdict) -> Self {
        self.add_verdict(threshold, verdict);
        self
    }
}

impl<G: Game> AbuseDetector<G> for SimpleRateLimiter {
    fn verify_game_input(&mut self, user: UserId, now: Instant, action: &G::InputAction) -> AbuseVerdict {
        let Some((min_duration, _)) = self.thresholds.last() else {
            return AbuseVerdict::Accept;
        };
        if let Some(last_instant) = self.last_action_time.get_mut(&user) {
            let duration = now - *last_instant;
            *last_instant = now;
            if duration > *min_duration {
                AbuseVerdict::Accept
            } else {
                let i = match self.thresholds.binary_search_by_key(&duration, |(t, _)| *t) {
                    Ok(i) => i,
                    Err(i) => i,
                };
                self.thresholds.get(i)
                    .map(|x| x.1.clone())
                    .unwrap_or(AbuseVerdict::Accept)
            }
        } else {
            self.last_action_time.insert(user, now);
            AbuseVerdict::Accept
        }
    }
}

/// Abuse detector for action-specific rate limiting.
///
/// Keeps track of when the given user last performed a given action kind
/// and can be configured to issue verdicts if the duration of time is below
/// specific thresholds. You can configure different thresholds and verdicts
/// for different action kinds.
///
/// You can add as many thresholds (and associated verdicts)
/// as you want, for any possible value of your action kind.
///
/// Will return the verdict associated with the lowest threshold
/// that the input duration is less than.
///
/// The "action kind" can be any type that impls `From<G::InputAction>`.
/// This gives you flexibility in how you categorize your game's possible
/// input actions.
#[derive(Clone)]
pub struct PerActionRateLimiter<Action: Eq + Hash> {
    thresholds: HashMap<Action, Vec<(Duration, AbuseVerdict)>>,
    last_action_time: HashMap<UserId, HashMap<Action, Instant>>,
}

impl<A: Eq + Hash> Default for PerActionRateLimiter<A> {
    fn default() -> Self {
        Self {
            thresholds: Default::default(),
            last_action_time: Default::default(),
        }
    }
}

impl<A: Eq + Hash + Clone> PerActionRateLimiter<A> {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn add_verdict(&mut self, action: &A, threshold: Duration, verdict: AbuseVerdict) {
        if let Some(thresholds) = self.thresholds.get_mut(action) {
            let i = match thresholds.binary_search_by_key(&threshold, |(t, _)| *t) {
                Ok(i) => i,
                Err(i) => i,
            };
            thresholds.insert(i, (threshold, verdict));
        } else {
            self.thresholds.insert(action.clone(), vec![(threshold, verdict)]);
        }
    }
    pub fn with_verdict(mut self, action: &A, threshold: Duration, verdict: AbuseVerdict) -> Self {
        self.add_verdict(action, threshold, verdict);
        self
    }
}

impl<A, G: Game> AbuseDetector<G> for PerActionRateLimiter<A>
where
    for<'g> A: Eq + Hash + From<&'g G::InputAction>,
{
    fn verify_game_input(&mut self, user: UserId, now: Instant, action: &G::InputAction) -> AbuseVerdict {
        let action = A::from(action);
        let Some(thresholds) = self.thresholds.get(&action) else {
            return AbuseVerdict::Accept;
        };
        let Some((min_duration, _)) = thresholds.last() else {
            return AbuseVerdict::Accept;
        };
        if let Some(map) = self.last_action_time.get_mut(&user) {
            if let Some(last_instant) = map.get_mut(&action) {
                let duration = now - *last_instant;
                *last_instant = now;
                if duration > *min_duration {
                    AbuseVerdict::Accept
                } else {
                    let i = match thresholds.binary_search_by_key(&duration, |(t, _)| *t) {
                        Ok(i) => i,
                        Err(i) => i,
                    };
                    thresholds.get(i)
                        .map(|x| x.1.clone())
                        .unwrap_or(AbuseVerdict::Accept)
                }
            } else {
                map.insert(action, now);
                AbuseVerdict::Accept
            }
        } else {
            let mut new = HashMap::default();
            new.insert(action, now);
            self.last_action_time.insert(user, new);
            AbuseVerdict::Accept
        }
    }
}

/// Start denying/punishing users after they have accumulated a high enough severity score.
///
/// This abuse detector wraps another abuse detector (the generic type parameter).
///
/// Will accumulate a total score per-user, every time the wrapped abuse detector
/// returns a `Flag` or `Deny` verdict.
///
/// After the total goes above specific configured thresholds, will start denying
/// all actions for that user, and optionally dealing out a punishment. The wrapped
/// abuse detector will still be called, but this wrapper will return `Deny` with
/// the configured punishment, no matter what it returns. Its severity score will
/// still be added to the total. This allows dealing out progressively more severe
/// punishments, if multiple thresholds are configured.
///
/// If the total is below any of the configured thresholds, this wrapper will return
/// whatever the inner type returned.
///
/// The severity value returned by this wrapper is the one returned by the inner
/// type, unmodified, *not* the accumulated total.
///
/// It is possible to optionally configure some forgiveness:
///  - a time duration after which the accumulator will reset
///  - an easing off factor: amount to remove from the total, per second elapsed time since the last action
///    (allowing a gradual cool-off)
///
/// These forgiveness factors will only be applied if the inner type
/// returns an `Accept` verdict.
#[derive(Clone)]
pub struct DenyPastSeverity<T> {
    detector: T,
    thresholds: Vec<(u32, Option<AbusePunishment>)>,
    accumulators: HashMap<UserId, (u32, Instant)>,
    threshold_reset: Option<Duration>,
    cooloff_rate: Option<u32>,
}

impl<T> DenyPastSeverity<T> {
    pub fn new(detector: T) -> Self {
        Self {
            detector,
            thresholds: Default::default(),
            accumulators: Default::default(),
            threshold_reset: None,
            cooloff_rate: None,
        }
    }
    pub fn set_threshold_reset(&mut self, threshold_reset: Option<Duration>) {
        self.threshold_reset = threshold_reset;
    }
    pub fn with_threshold_reset(mut self, threshold_reset: Option<Duration>) -> Self {
        self.set_threshold_reset(threshold_reset);
        self
    }
    pub fn set_cooloff_rate(&mut self, cooloff_rate: Option<u32>) {
        self.cooloff_rate = cooloff_rate;
    }
    pub fn with_cooloff_rate(mut self, cooloff_rate: Option<u32>) -> Self {
        self.set_cooloff_rate(cooloff_rate);
        self
    }
    pub fn add_punishment(&mut self, threshold: u32, punishment: Option<AbusePunishment>) {
        let i = match self.thresholds.binary_search_by_key(&threshold, |(t, _)| *t) {
            Ok(i) => i,
            Err(i) => i,
        };
        self.thresholds.insert(i, (threshold, punishment));
    }
    pub fn with_punishment(mut self, threshold: u32, punishment: Option<AbusePunishment>) -> Self {
        self.add_punishment(threshold, punishment);
        self
    }
    pub fn reset_user(&mut self, user: UserId) {
        self.accumulators.remove(&user);
    }
    pub fn reset_all(&mut self) {
        self.accumulators.clear();
    }
    pub fn score(&self, user: UserId) -> u32 {
        self.accumulators.get(&user)
            .map(|(t, _)| *t)
            .unwrap_or(0)
    }
    pub fn subtract_score(&mut self, user: UserId, cooloff_amount: u32) -> u32 {
        if let Some((total, _)) = self.accumulators.get_mut(&user) {
            if cooloff_amount >= *total {
                self.accumulators.remove(&user);
                0
            } else {
                *total -= cooloff_amount;
                *total
            }
        } else {
            0
        }
    }
}

impl<G: Game, T: AbuseDetector<G>> AbuseDetector<G> for DenyPastSeverity<T> {
    fn verify_game_input(&mut self, user: UserId, now: Instant, action: &G::InputAction) -> AbuseVerdict {
        let verdict = self.detector.verify_game_input(user, now, action);
        let Some((first_threshold, _)) = self.thresholds.first() else {
            return verdict;
        };
        if let Some((total, last_instant)) = self.accumulators.get_mut(&user) {
            let duration = now - *last_instant;
            *last_instant = now;
            match verdict {
                AbuseVerdict::Accept => {
                    if let Some(threshold_reset) = self.threshold_reset {
                        if duration > threshold_reset {
                            self.accumulators.remove(&user);
                            return verdict;
                        }
                    }
                    if let Some(cooloff_rate) = self.cooloff_rate {
                        let cooloff_amount = (cooloff_rate as f32 * duration.as_secs_f32()) as u32;
                        if cooloff_amount >= *total {
                            self.accumulators.remove(&user);
                            return verdict;
                        } else {
                            *total -= cooloff_amount;
                        }
                    }
                }
                AbuseVerdict::Flag { severity } => {
                    *total += severity;
                }
                AbuseVerdict::Deny { severity, .. } => {
                    *total += severity;
                }
            };
            if *total < *first_threshold {
                verdict
            } else {
                if let Some((_, punishment)) = self.thresholds.iter()
                    .rfind(|(t, _)| *total >= *t)
                {
                    AbuseVerdict::Deny {
                        severity: verdict.severity(),
                        punishment: punishment.clone(),
                    }
                } else {
                    verdict
                }
            }
        } else {
            match verdict {
                AbuseVerdict::Accept => {},
                AbuseVerdict::Flag { severity } => {
                    self.accumulators.insert(user, (severity, now));
                }
                AbuseVerdict::Deny { severity, .. } => {
                    self.accumulators.insert(user, (severity, now));
                }
            };
            verdict
        }
    }
}
