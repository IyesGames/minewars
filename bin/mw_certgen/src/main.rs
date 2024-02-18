use anyhow::{Result as AnyResult, Context};
use clap::{Parser, Subcommand};
use rcgen::{BasicConstraints, Certificate, CertificateParams, IsCa, KeyPair, SanType};

use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(about = "Generates TLS certificates for MineWars servers and clients.")]
struct Cli {
    #[command(subcommand)]
    command: CliCommand,
    #[arg(short, long)]
    dir: Option<PathBuf>,
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand, Debug)]
enum CliCommand {
    /// Generate a root CA certificate.
    GenRootCa(GenRootCaArgs),
    /// Generate a limited CA certificate.
    /// For example, if we want to have a CA for a swarm of Host servers, instead of having
    /// them under the root CA directly.
    GenSubCa(GenSubCaArgs),
    /// Generate a CA certificate to use for session authentication.
    /// This will be used by the Auth server (or RPC) to generate single-use certificates
    /// for players upon handoff to a Host server that has per-session player authentication enabled.
    GenSessionCa(GenSessionCaArgs),
    /// Generate a client certificate for use by players when connecting to a Host server
    /// that does not have per-session player authentication.
    GenHostClientCert(GenHostClientCertArgs),
    /// Generate a client certificate for use by players when connecting to an Auth server.
    GenAuthClientCert(GenAuthClientCertArgs),
    /// Generate a server certificate for use by a Host server.
    GenHostServerCert(GenHostServerCertArgs),
    /// Generate a server certificate for use by an Auth server.
    GenAuthServerCert(GenAuthServerCertArgs),
    /// Generate a client certificate for use by a Host server when connecting to an Auth server.
    /// (for the HostAuth protocol)
    GenHostAuthClientCert(GenHostAuthClientCertArgs),
    /// Generate a server certificate for use an Auth server for HostAuth connections.
    GenHostAuthServerCert(GenHostAuthServerCertArgs),
    /// Generate a client certificate for RPC control of a Host server.
    GenHostRpcClientCert(GenHostRpcClientCertArgs),
    /// Generate a server certificate for a Host server's RPC interface.
    GenHostRpcServerCert(GenHostRpcServerCertArgs),
}

#[derive(Parser, Debug)]
struct GenRootCaArgs {
    #[command(flatten)]
    out: CommonOutArgs,
}

#[derive(Parser, Debug)]
struct GenSubCaArgs {
    #[command(flatten)]
    out: CommonOutArgs,
    #[command(flatten)]
    ca: CommonCaArgs,
}

#[derive(Parser, Debug)]
struct GenSessionCaArgs {
    #[command(flatten)]
    out: CommonOutArgs,
    #[command(flatten)]
    ca: CommonCaArgs,
}

#[derive(Parser, Debug)]
struct GenHostServerCertArgs {
    #[command(flatten)]
    san: CommonSanArgs,
    #[command(flatten)]
    out: CommonOutArgs,
    #[command(flatten)]
    ca: CommonCaArgs,
}

#[derive(Parser, Debug)]
struct GenAuthServerCertArgs {
    #[command(flatten)]
    san: CommonSanArgs,
    #[command(flatten)]
    out: CommonOutArgs,
    #[command(flatten)]
    ca: CommonCaArgs,
}

#[derive(Parser, Debug)]
struct GenHostAuthServerCertArgs {
    #[command(flatten)]
    san: CommonSanArgs,
    #[command(flatten)]
    out: CommonOutArgs,
    #[command(flatten)]
    ca: CommonCaArgs,
}

#[derive(Parser, Debug)]
struct GenHostRpcServerCertArgs {
    #[command(flatten)]
    san: CommonSanArgs,
    #[command(flatten)]
    out: CommonOutArgs,
    #[command(flatten)]
    ca: CommonCaArgs,
}

#[derive(Parser, Debug)]
struct GenHostAuthClientCertArgs {
    #[command(flatten)]
    out: CommonOutArgs,
    #[command(flatten)]
    ca: CommonCaArgs,
}

#[derive(Parser, Debug)]
struct GenHostRpcClientCertArgs {
    #[command(flatten)]
    out: CommonOutArgs,
    #[command(flatten)]
    ca: CommonCaArgs,
}

#[derive(Parser, Debug)]
struct GenHostClientCertArgs {
    #[command(flatten)]
    out: CommonOutArgs,
    #[command(flatten)]
    ca: CommonCaArgs,
}

#[derive(Parser, Debug)]
struct GenAuthClientCertArgs {
    #[command(flatten)]
    out: CommonOutArgs,
    #[command(flatten)]
    ca: CommonCaArgs,
}

#[derive(Parser, Debug)]
struct CommonSanArgs {
    /// Server DNS Name(s), if any should be used for verification
    #[arg(short, long)]
    name: Vec<String>,
    /// Server IP Address(es), if any should be used for verification
    #[arg(long)]
    ip: Vec<String>,
    /// Server URI(s), if any should be used for verification
    #[arg(short, long)]
    uri: Vec<String>,
}

#[derive(Parser, Debug)]
struct CommonCaArgs {
    /// Parent CA cert to sign the new certificate with
    #[arg(long)]
    ca: PathBuf,
    /// Parent CA key to sign the new certificate with
    #[arg(long)]
    ca_key: PathBuf,
}

#[derive(Parser, Debug)]
struct CommonOutArgs {
    /// File name for the certificate
    out_cert_file: PathBuf,
    /// File name for the private key
    out_key_file: PathBuf,
}

impl CommonSanArgs {
    fn add_to_params(&self, params: &mut CertificateParams) -> AnyResult<()> {
        for uri in &self.uri {
            params.subject_alt_names
                .push(SanType::URI(uri.clone()));
        }
        for dns in &self.name {
            params.subject_alt_names
                .push(SanType::DnsName(dns.clone()));
        }
        for ip in &self.ip {
            params.subject_alt_names
                .push(SanType::DnsName(ip.parse()?));
        }
        Ok(())
    }
}

impl Cli {
    fn run(&self) -> AnyResult<()> {
        if let Some(dir) = &self.dir {
            std::fs::create_dir_all(dir)
                .context(format!("Directory {:?} is not accessible and cannot be created", dir))?;
            std::env::set_current_dir(dir)
                .context(format!("Failed to change directory to {:?}", dir))?;
        }
        match &self.command {
            CliCommand::GenRootCa(args) => {
                pipeline(&args.out, None, gen_root_ca, ())?;
            }
            CliCommand::GenSubCa(args) => {
                pipeline(&args.out, Some(&args.ca), gen_sub_ca, ())?;
            }
            CliCommand::GenSessionCa(args) => {
                pipeline(&args.out, Some(&args.ca), gen_session_ca, ())?;
            }
            CliCommand::GenHostClientCert(args) => {
                pipeline(&args.out, Some(&args.ca), gen_host_client_cert, ())?;
            }
            CliCommand::GenAuthClientCert(args) => {
                pipeline(&args.out, Some(&args.ca), gen_auth_client_cert, ())?;
            }
            CliCommand::GenHostServerCert(args) => {
                pipeline(&args.out, Some(&args.ca), gen_host_server_cert, &args.san)?;
            }
            CliCommand::GenAuthServerCert(args) => {
                pipeline(&args.out, Some(&args.ca), gen_auth_server_cert, &args.san)?;
            }
            CliCommand::GenHostAuthClientCert(args) => {
                pipeline(&args.out, Some(&args.ca), gen_hostauth_client_cert, ())?;
            }
            CliCommand::GenHostAuthServerCert(args) => {
                pipeline(&args.out, Some(&args.ca), gen_hostauth_server_cert, &args.san)?;
            }
            CliCommand::GenHostRpcClientCert(args) => {
                pipeline(&args.out, Some(&args.ca), gen_hostrpc_client_cert, ())?;
            }
            CliCommand::GenHostRpcServerCert(args) => {
                pipeline(&args.out, Some(&args.ca), gen_hostrpc_server_cert, &args.san)?;
            }
        }
        Ok(())
    }
}

fn pipeline<T>(
    out: &CommonOutArgs,
    signer: Option<&CommonCaArgs>,
    gen: impl FnOnce(T) -> AnyResult<Certificate>,
    args: T,
) -> AnyResult<()> {
    let cert = gen(args)
        .context("Failed to generate certificate")?;
    if let Some(signer) = signer {
        let ca = input_ca_pair(&signer.ca, &signer.ca_key)
            .context("Failed to load CA certificate and key")?;
        output_pair_signed(&ca, &cert, &out.out_cert_file, &out.out_key_file)
            .context("Failed to output signed certificate and key")?;
    } else {
        output_pair(&cert, &out.out_cert_file, &out.out_key_file)
            .context("Failed to output certificate and key")?;
    }
    Ok(())
}

fn input_ca_pair(in_cert_file: &Path, in_key_file: &Path) -> AnyResult<Certificate> {
    let cert_bytes = std::fs::read(in_cert_file)?;
    let key_bytes = std::fs::read(in_key_file)?;
    let key_pair = KeyPair::from_der(&key_bytes)?;
    let cert_params = CertificateParams::from_ca_cert_der(&cert_bytes, key_pair)?;
    Ok(Certificate::from_params(cert_params)?)
}

fn output_pair(cert: &Certificate, out_cert_file: &Path, out_key_file: &Path) -> AnyResult<()> {
    std::fs::write(out_cert_file, cert.serialize_der()?)?;
    std::fs::write(out_key_file, cert.serialize_private_key_der())?;
    Ok(())
}

fn output_pair_signed(signer: &Certificate, cert: &Certificate, out_cert_file: &Path, out_key_file: &Path) -> AnyResult<()> {
    std::fs::write(out_cert_file, cert.serialize_der_with_signer(signer)?)?;
    std::fs::write(out_key_file, cert.serialize_private_key_der())?;
    Ok(())
}

fn gen_root_ca(_: ()) -> AnyResult<Certificate> {
    let mut params = CertificateParams::default();
    params.is_ca = IsCa::Ca(BasicConstraints::Constrained(0));
    Ok(Certificate::from_params(params)?)
}

fn gen_sub_ca(_: ()) -> AnyResult<Certificate> {
    let mut params = CertificateParams::default();
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    Ok(Certificate::from_params(params)?)
}

fn gen_session_ca(_: ()) -> AnyResult<Certificate> {
    let mut params = CertificateParams::default();
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    Ok(Certificate::from_params(params)?)
}

fn gen_host_server_cert(san: &CommonSanArgs) -> AnyResult<Certificate> {
    let mut params = CertificateParams::default();
    params.is_ca = IsCa::ExplicitNoCa;
    san.add_to_params(&mut params)?;
    Ok(Certificate::from_params(params)?)
}

fn gen_auth_server_cert(san: &CommonSanArgs) -> AnyResult<Certificate> {
    let mut params = CertificateParams::default();
    params.is_ca = IsCa::ExplicitNoCa;
    san.add_to_params(&mut params)?;
    Ok(Certificate::from_params(params)?)
}

fn gen_hostauth_server_cert(san: &CommonSanArgs) -> AnyResult<Certificate> {
    let mut params = CertificateParams::default();
    params.is_ca = IsCa::ExplicitNoCa;
    san.add_to_params(&mut params)?;
    Ok(Certificate::from_params(params)?)
}

fn gen_hostrpc_server_cert(san: &CommonSanArgs) -> AnyResult<Certificate> {
    let mut params = CertificateParams::default();
    params.is_ca = IsCa::ExplicitNoCa;
    san.add_to_params(&mut params)?;
    Ok(Certificate::from_params(params)?)
}

fn gen_host_client_cert(_: ()) -> AnyResult<Certificate> {
    let mut params = CertificateParams::default();
    params.is_ca = IsCa::ExplicitNoCa;
    Ok(Certificate::from_params(params)?)
}

fn gen_auth_client_cert(_: ()) -> AnyResult<Certificate> {
    let mut params = CertificateParams::default();
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    Ok(Certificate::from_params(params)?)
}

fn gen_hostauth_client_cert(_: ()) -> AnyResult<Certificate> {
    let mut params = CertificateParams::default();
    params.is_ca = IsCa::ExplicitNoCa;
    Ok(Certificate::from_params(params)?)
}

fn gen_hostrpc_client_cert(_: ()) -> AnyResult<Certificate> {
    let mut params = CertificateParams::default();
    params.is_ca = IsCa::ExplicitNoCa;
    Ok(Certificate::from_params(params)?)
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = cli.run() {
        eprintln!("Error: {:#}", e);
    }
}
