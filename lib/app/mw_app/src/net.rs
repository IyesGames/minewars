use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.init_resource::<NetInfo>();
}

#[derive(Resource, Default)]
pub struct NetInfo {
    pub rtt: Option<Duration>,
    pub server_addr: Option<SocketAddr>,
}
