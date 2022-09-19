use mw_common::grid::*;
use mw_common::plid::PlayerId;

use crate::prelude::*;

/// Implements switching between multiple player perspectives
///
/// The actual functionality is gated behind the existence of a
/// `Views` resource. The game mode code is expected to set that
/// up, if a multi-view session is desired. If it is missing,
/// this plugin effectively does nothing.
///
///
pub struct MapViewsPlugin;

impl Plugin for MapViewsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ViewSwitchEvent>();
    }
}

/// Resource indicating the PlayerId of the currently visible view
///
/// View switching triggered by ViewSwitchEvent.
pub struct ActiveView(PlayerId);

impl ActiveView {
    /// Accessor, to keep the plid externally read-only (not pub)
    pub fn plid(&self) -> PlayerId {
        self.0
    }
}

/// Event for triggering a switch to another view
pub struct ViewSwitchEvent {
    pub plid: PlayerId,
}

struct ViewTile {
    /// bits 2..0 = PlayerId
    /// bits 5..3 = Digit
    owner_digit: u8,
}

impl ViewTile {
    const MASK_PLID: u8   = 0b00000111;
    const SHIFT_PLID: u8  = 0;
    const MASK_DIGIT: u8  = 0b00111000;
    const SHIFT_DIGIT: u8 = 3;

    fn owner(&self) -> PlayerId {
        PlayerId::from((self.owner_digit & Self::MASK_PLID) >> Self::SHIFT_PLID)
    }

    fn set_owner(&mut self, plid: PlayerId) {
        let plid = u8::from(plid) << Self::SHIFT_PLID;
        self.owner_digit =
            (self.owner_digit & !Self::MASK_PLID) |
            (plid & Self::MASK_PLID);
    }

    fn digit(&self) -> u8 {
        (self.owner_digit & Self::MASK_DIGIT) >> Self::SHIFT_DIGIT
    }

    fn set_digit(&mut self, digit: u8) {
        let digit = (digit as u8) << Self::SHIFT_DIGIT;
        self.owner_digit =
            (self.owner_digit & !Self::MASK_DIGIT) |
            (digit & Self::MASK_DIGIT);
    }
}

/// A single "view"
///
/// Cache of the game state, from a specific player's point of view.
/// Allows the game client to switch between displaying the
/// game as seen by different players.
///
/// In a normal game, there is only one view (the active player's).
///
/// In a spectator session, there is one for each player in the game.
///
/// This is a compact representation of the game state, kept up-to-date
/// with map events. Upon view switch, all the data is populated into
/// the entities/components of all the respective map tiles.
struct View {
    map: MapAny<ViewTile>,
}

/// Resource type for storing all the view data
struct Views {
    /// in order by PlayerId, use PlayerId to index
    views: Vec<View>,
    /// mapping for navigation; these views can be toggled by the user
    navmap: Vec<PlayerId>,
}

fn view_switcher(
    evr: EventReader<ViewSwitchEvent>,
    views: Res<Views>,
) {
}
