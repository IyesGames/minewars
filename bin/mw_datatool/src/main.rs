use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::prelude::*;

mod prelude {
    pub use anyhow::{Result as AnyResult, Context, bail};
}

mod cmd {
    pub mod info;
    pub mod gen_map;
    pub mod map_ascii;
    pub mod checksum_verify;
    pub mod checksum_fix;
    pub mod reencode;
}

#[derive(Parser, Debug)]
#[command(about = "Tool for working with MineWars protocol data files and streams.")]
struct Cli {
    #[command(flatten)]
    common: CommonArgs,
    #[command(subcommand)]
    command: CliCommand,
}

#[derive(Parser, Debug)]
struct CommonArgs {
    /// Input file
    #[arg(short, long)]
    input: Option<PathBuf>,
    /// Output file
    #[arg(short, long)]
    output: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
enum CliCommand {
    /// Show info about the header and general layout of the file
    Info(InfoArgs),
    /// Generate a new MineWars map
    GenMap(GenMapArgs),
    /// Read the map data from a file and display it as ascii art
    MapAscii(MapAsciiArgs),
    /// Analyze the file's encoding and show technical statistics
    Analyze(AnalyzeArgs),
    /// Remove parts of the file
    Strip(StripArgs),
    /// Extract the game rules from a MineWars file into human-friendly TOML format
    RulesMw2toml(RulesMw2tomlArgs),
    /// Replace the game rules in a MineWars file with new values from human-friendly TOML format
    RulesToml2mw(RulesToml2mwArgs),
    /// Verify a MineWars file's checksums
    ChecksumVerify(ChecksumVerifyArgs),
    /// Recompute a MineWars file's checksums
    ChecksumFix(ChecksumFixArgs),
    /// Re-encode the file, possibly with different compression
    Reencode(ReencodeArgs),
    /// Disassemble frame data
    Disasm(DisasmArgs),
    /// Assemble frame data
    Asm(AsmArgs),
}

#[derive(Parser, Debug)]
struct InfoArgs {
    /// Do not verify the checksums of the input file
    #[arg(long)]
    ignore_checksums: bool,
}

#[derive(Parser, Debug)]
struct GenMapArgs {
    /// Specify a RNG seed to use
    #[arg(long)]
    seed: Option<u64>,
    /// 0-255: small values = less land / more water, big values = more land / less water
    #[arg(long)]
    land_bias: Option<u64>,
    /// The map size
    #[arg(short, long)]
    #[arg(value_parser = clap::value_parser!(u8).range(1..=125))]
    size: u8,
    /// Generate this many cities/regions
    #[arg(short, long)]
    cits: u8,
    /// Also display the newly-generated map as ascii art
    #[arg(short, long)]
    ascii: bool,
    /// Also generate Mines (provide a value 0-255 indicating density/probability)
    #[arg(short, long)]
    mines: Option<u8>,
    /// Also generate Decoys (provide a value 0-255 indicating probability of a mine being turned into a decoy)
    #[arg(short, long)]
    decoys: Option<u8>,
}

#[derive(Parser, Debug)]
struct MapAsciiArgs {
}

#[derive(Parser, Debug)]
struct AnalyzeArgs {
}

#[derive(Parser, Debug)]
struct StripArgs {
    /// Strip player info (names)
    #[arg(short, long)]
    players: bool,
    /// Strip all frames (gameplay data)
    #[arg(short, long)]
    frames: bool,
    /// Strip frames after the given timestamp (milliseconds)
    #[arg(long)]
    frames_after_ms: u64,
    /// Strip all map data; also enables `frames`
    #[arg(short, long)]
    map: bool,
    /// Strip cities and regions from map; also enables `frames`
    #[arg(short, long)]
    cits: bool,
    /// Strip game rules
    #[arg(short, long)]
    rules: bool,
    /// Strip everything except the map data and cities/regions
    /// (enables: players, frames, rules)
    #[arg(long)]
    mapcitonly: bool,
    /// Strip everything except the map data
    /// (enables: players, frames, rules, cits)
    #[arg(long)]
    maponly: bool,
    /// Strip everything except the game rules
    /// (enables: players, frames, map, cits)
    #[arg(long)]
    rulesonly: bool,
    /// Strip everything except the player info (names)
    /// (enables: frames, rules, map, cits)
    #[arg(long)]
    playersonly: bool,
}

#[derive(Parser, Debug)]
struct RulesMw2tomlArgs {
}

#[derive(Parser, Debug)]
struct RulesToml2mwArgs {
    /// Do not error out if some values cannot be represented exactly, just approximate them
    #[arg(short, long)]
    lossy: bool,
}

#[derive(Parser, Debug)]
struct ChecksumVerifyArgs {
}

#[derive(Parser, Debug)]
struct ChecksumFixArgs {
}

#[derive(Parser, Debug)]
struct ReencodeArgs {
    /// Do not verify the checksums of the input file
    #[arg(long)]
    ignore_checksums: bool,
    /// Compress the Map Data (default is to keep as-is)
    #[arg(long)]
    compress_map: bool,
    /// Decompress the Map Data (default is to keep as-is)
    #[arg(long)]
    decompress_map: bool,
    /// Compress the Frames Data (default is to keep as-is)
    #[arg(long)]
    compress_frames: bool,
    /// Decompress the Frames Data (default is to keep as-is)
    #[arg(long)]
    decompress_frames: bool,
    /// Anonymize (strip player names)
    #[arg(short = 'a', long)]
    anonymize: bool,
}

#[derive(Parser, Debug)]
struct DisasmArgs {
    /// Timestamp (milliseconds) to start from (default is the very beginning)
    #[arg(short, long)]
    start_time: Option<u64>,
    /// Timestamp (milliseconds) to end at (default is the very end)
    #[arg(short, long)]
    end_time: Option<u64>,
    /// Only disassemble at most this many frames
    #[arg(short, long)]
    n_frames: Option<u64>,
}

#[derive(Parser, Debug)]
struct AsmArgs {
    /// Timestamp (milliseconds) where to insert the new data (default is at the end)
    #[arg(short, long)]
    time: Option<u64>,
    /// Only apply these PlayerIds (option may be repeated)
    #[arg(short, long)]
    plid: Vec<u8>,
    /// Overwrite existing data if frames overlap (default is to append to existing frames)
    #[arg(short, long)]
    replace: bool,
}

impl Cli {
    fn run(&self) -> AnyResult<()> {
        match &self.command {
            CliCommand::Info(args) => crate::cmd::info::main(&self.common, &args),
            CliCommand::GenMap(args) => crate::cmd::gen_map::main(&self.common, &args),
            CliCommand::MapAscii(args) => crate::cmd::map_ascii::main(&self.common, &args),
            CliCommand::Analyze(args) => todo!(),
            CliCommand::Strip(args) => todo!(),
            CliCommand::RulesMw2toml(args) => todo!(),
            CliCommand::RulesToml2mw(args) => todo!(),
            CliCommand::ChecksumVerify(args) => crate::cmd::checksum_verify::main(&self.common, &args),
            CliCommand::ChecksumFix(args) => crate::cmd::checksum_fix::main(&self.common, &args),
            CliCommand::Reencode(args) => crate::cmd::reencode::main(&self.common, &args),
            CliCommand::Disasm(args) => todo!(),
            CliCommand::Asm(args) => todo!(),
        }
    }
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = cli.run() {
        eprintln!("Error: {:#}", e);
    }
}
