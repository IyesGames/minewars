// use mw_common::net::{ClientSettings, ServerSettings};

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    // app.init_setting::<NetworkingSettings>(SETTINGS_LOCAL.as_ref());
}

// #[derive(Reflect, Clone, PartialEq)]
// #[reflect(Setting)]
// pub struct NetworkingSettings {
//     pub enabled: bool,
//     pub threads: usize,
//     pub my_addr: String,
//     pub default_client_settings: Option<ClientSettings>,
//     pub server_settings: Option<ServerSettings>,
// }

// impl Setting for NetworkingSettings {}

// impl Default for NetworkingSettings {
//     fn default() -> Self {
//         let threads = if num_cpus::get_physical() > 4 {
//             4
//         } else {
//             2
//         };
//         Self {
//             enabled: true,
//             threads,
//             my_addr: "0.0.0.0:13370".into(),
//             server_settings: Some(ServerSettings {
//                 server_certs: vec![
//                     "cfg/cert/hostclient.cert.der".into(),
//                     "cfg/cert/apps.ca.cert.der".into(),
//                 ],
//                 server_key: "cfg/cert/hostclient.key.der".into(),
//                 client_ca: vec![
//                     "cfg/cert/apps.ca.cert.der".into(),
//                 ],
//             }),
//             default_client_settings: Some(ClientSettings {
//                 client_certs: vec![
//                     "cfg/cert/hostclient.cert.der".into(),
//                     "cfg/cert/apps.ca.cert.der".into(),
//                 ],
//                 client_key: Some("cfg/cert/hostclient.key.der".into()),
//                 server_ca: vec![
//                     "cfg/cert/apps.ca.cert.der".into(),
//                     "cfg/cert/hosts.ca.cert.der".into(),
//                 ],
//             }),
//         }
//     }
// }
