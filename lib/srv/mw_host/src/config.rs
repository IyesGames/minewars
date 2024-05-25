use crate::prelude::*;

use mw_common::net::ControlListMode;
use mw_proto_hostrpc::RpcMethodName;

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct Config {
    /// General options that govern everything
    pub general: GeneralConfig,
    /// Options related to the Game Server (for incoming player connections)
    pub server: ServerConfig,
    /// Options related to the RPC Server (for administration / management)
    pub rpc: RpcConfig,
    /// Options related to the HostAuth Client (for connecting to a management/account service)
    pub hostauth: HostAuthConfig,
    /// Configuration of preset game sessions to be hosted on the server.
    /// Optional. Additional sessions can be created at runtime via RPC or HostAuth.
    pub sessions: SessionConfig,
}

/// General options, regardless of protocol
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct GeneralConfig {
    pub log_file: Option<PathBuf>,
    pub log_debug: bool,
}

/// Settings for automatic session creation
/// (useful if running without RPC or HostAuth)
#[derive(Debug, Clone, Default)]
#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct SessionConfig {
    /// Limit how many concurrent sessions the server is allowed to host.
    pub max_sessions: Option<u32>,
    /// Create a fixed number of sessions with the provided settings.
    pub preset: HashMap<String, SessionParams>,
    /// If set, a new session will be created automatically with the given preset,
    /// if new players connect and all other sessions are full.
    pub autosession: Option<String>,
    /// Automatically create N sessions of each specified preset, at startup.
    pub autostart: HashMap<String, u32>,
}

/// Settings for a specific game session
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct SessionParams {
    /// The game to be played in this session
    pub mode: GameMode,
    /// Should the session be recreated/restarted after game over?
    pub autorestart: bool,
    /// true: The game can start with fewer players, new players can join mid-game.
    /// false: The game requires all players to connect at the start, no new players can join mid-game.
    pub open_session: bool,
    /// Should spectators be allowed for this session?
    pub allow_spectators: Option<bool>,
    /// Number of logical players for the game
    pub n_plids: u8,
    /// Number of actual clients controlling each logical player
    pub n_subplids: u8,
    /// What map will this session be played on?
    #[serde(flatten)]
    pub map: MapParams,
}

/// Settings for map generation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize)]
#[serde(tag = "map_mode")]
pub enum MapParams {
    File {
        map_path: PathBuf,
    },
    Generate {
        map_topology: mw_common::grid::Topology,
        map_style: MapStyle,
        map_seed: Option<u64>,
        map_size: u8,
        map_n_cits: u8,
        map_land_bias: u8,
    },
}

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub enum GameMode {
    Minesweeper,
    MineWars,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize)]
pub enum MapStyle {
    Flat,
    MineWars,
}

/// Confguration for the Host Server Protocol
///
/// This is where you configure everything related to the games/sessions to be hosted on the server,
/// and the players/clients who will connect to play or spectate.
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct ServerConfig {
    /// What IP(s) to listen for player connections on
    pub listen_players: HashSet<SocketAddr>,
    /// Our TLS Server certificate chain (DER format)
    pub cert: Vec<PathBuf>,
    /// Our TLS Server key (DER format)
    pub key: PathBuf,
    /// Mode for IP restriction
    pub ip_control: ControlListMode,
    /// List of IPs for IP restriction
    pub ip_list: IpListOrFile,
    /// If players connect with client authentication, expect certs signed by this CA
    pub player_ca: PathBuf,
    /// Allow players to connect without a prior `ExpectPlayer` from RPC/hostauth
    pub allow_players_unexpected: bool,
    /// Allow players to connect without a client TLS certificate (disable client cert verification)
    pub allow_players_nocert: bool,
    /// Allow players to connect from an IP other than the one specified by `ExpectPlayer`
    pub allow_players_anyip: bool,
    /// Accept players that want the server to assign them a session (not connecting for a specific session)
    pub allow_anysession: bool,
    /// Global toggle for enabling/disabling spectator mode. Can also be controlled per-session.
    pub allow_spectators: bool,
}

/// Confguration for the HostAuth Client
///
/// This is where you configure any (optional) connection to an Auth Server.
///
/// If enabled, this Host Server will connect to the configured Auth Server,
/// to allow the Auth Server to manage it.
///
/// HostAuth is basically "reverse-RPC". RPC lets other software connect to us
/// to control us. HostAuth is us connecting to something that will control us.
///
/// Given that we are the ones connecting to something known and pre-configured,
/// HostAuth can be a more secure way of managing the server than RPC.
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct HostAuthConfig {
    /// Enable HostAuth
    pub enable: bool,
    /// The HostAuth Server to connect to
    pub server: SocketAddr,
    /// Our TLS Client certificate chain (DER format)
    pub cert: Vec<PathBuf>,
    /// Our TLS Client key (DER format))
    pub key: PathBuf,
    /// What payload formats do we accept
    pub allow_payloads: HashSet<PayloadKind>,
    /// Mode for resticting the available RPC methods
    pub rpc_method_control: ControlListMode,
    /// List of RPC methods to be restricted
    pub rpc_methods_list: HashSet<RpcMethodName>,
}

/// Configruation for the RPC Server
///
/// RPC is a mechanism that allows external tools to connect to this server
/// to control and configure it.
///
/// This is security sensitive and should probably be severely restricted
/// to your local machine or network, or disabled altogether.
///
/// For production deployments, prefer using HostAuth instead of RPC.
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct RpcConfig {
    /// Enable RPC
    pub enable: bool,
    /// What IP(s) to listen for RPC connections on
    pub listen: HashSet<SocketAddr>,
    /// Our TLS Server certificate chain (DER format)
    pub cert: Vec<PathBuf>,
    /// Our TLS Server key (DER format)
    pub key: PathBuf,
    /// Enable TLS certificate verification of clients.
    pub require_client_cert: bool,
    /// If enabled, require clients to have a certificate signed by the CA provided here.
    pub client_ca: PathBuf,
    /// Mode for IP restriction
    pub ip_control: ControlListMode,
    /// List of IPs for IP restriction
    pub ip_list: IpListOrFile,
    /// What payload formats do we accept
    pub allow_payloads: HashSet<PayloadKind>,
    /// Mode for resticting the available RPC methods
    pub rpc_method_control: ControlListMode,
    /// List of RPC methods to be restricted
    pub rpc_methods_list: HashSet<RpcMethodName>,
}

/// Payload formats that can be accepted over our various protocols.
///
/// Payloads are additional data sent alongside a protocol message,
/// if any such data is required for the operation to be performed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize)]
pub enum PayloadKind {
    /// MineWars Replay/Scenario File Format
    Minewars,
    /// MineWars Game Rules/Config encoded as TOML
    TomlRules,
}

/// Helper for configuring an IP restriction list
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum IpListOrFile {
    /// IPs listed inline in the main config file
    List(HashSet<IpAddr>),
    /// IPs listed in a separate file, newline-delimited
    File(PathBuf),
}

impl IpListOrFile {
    pub fn temporary_todo_unwrap(&self) -> &HashSet<IpAddr> {
        // TODO: this function exists because we haven't implemented support for IP List Files yet
        match self {
            IpListOrFile::List(x) => &x,
            IpListOrFile::File(_) => unimplemented!(),
        }
    }
}

impl Config {
    /// Check for any CLI Args that override config options and modify the config accordingly.
    pub fn apply_cli(&mut self, args: &crate::cli::Args) {
        self.general.log_debug |= args.debug;
        if let Some(log_file) = &args.log {
            self.general.log_file = Some(log_file.clone());
        }
    }
    pub fn reparent_paths(&mut self, config_path: &Path) {
        let dir = if config_path.is_dir() {
            config_path
        } else {
            let Some(parent) = config_path.parent() else {
                return;
            };
            parent
        };
        self.rpc.key = reparent_path(dir, &self.rpc.key);
        self.rpc.client_ca = reparent_path(dir, &self.rpc.client_ca);
        for path in &mut self.rpc.cert {
            *path = reparent_path(dir, path);
        }
        self.server.key = reparent_path(dir, &self.server.key);
        self.server.player_ca = reparent_path(dir, &self.server.player_ca);
        for path in &mut self.server.cert {
            *path = reparent_path(dir, path);
        }
        self.hostauth.key = reparent_path(dir, &self.hostauth.key);
        for path in &mut self.hostauth.cert {
            *path = reparent_path(dir, path);
        }
    }
}

fn reparent_path(dir: &Path, path: &Path) -> PathBuf {
    if path.is_relative() {
        dir.join(path)
    } else {
        path.to_owned()
    }
}
