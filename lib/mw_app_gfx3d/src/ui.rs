use crate::prelude::*;

pub mod perf;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins((
        self::perf::plugin,
    ));
}
