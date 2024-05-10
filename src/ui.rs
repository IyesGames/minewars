use mw_app::prelude::*;

mod perf;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        perf::plugin,
    ));
}
