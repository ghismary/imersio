use clap::Parser;
use imersio_sip::{SipUri, Transport};
use std::path::PathBuf;
use std::process::ExitCode;
use tokio::net::{TcpListener, UdpSocket};
use tokio::select;
use tracing::{debug, error, info, warn};

mod config;

use config::Config;

const DEFAULT_SIP_PORT: u16 = 5060;
const DEFAULT_SIPS_PORT: u16 = 5061;
const DEFAULT_CONFIG_PATH: &str = "/etc/imersio/imersio.toml";

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct MainArgs {
    #[arg(short = 'c', long, default_value = DEFAULT_CONFIG_PATH)]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> ExitCode {
    let args = MainArgs::parse();
    let config: Config = match args.config.as_path().try_into() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("{err}");
            return ExitCode::FAILURE;
        }
    };

    tracing_subscriber::fmt()
        .compact()
        .with_max_level(Into::<tracing::Level>::into(config.proxy.log_level))
        .with_target(false)
        .init();

    info!(
        "Starting {} {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    match run(config).await {
        Ok(()) => {
            info!("Shutting down...");
            ExitCode::SUCCESS
        }
        Err(err) => {
            error!("{err}");
            ExitCode::FAILURE
        }
    }
}

#[tracing::instrument]
async fn run(config: Config) -> Result<(), std::io::Error> {
    let transports = config::transports_by_ip_type(config.proxy.transports);
    for transport_by_ip_type in transports {
        let mut bind_done_for_ipv6 = false;
        if let Some(sip_uri) = transport_by_ip_type.ipv6 {
            enable_transport(&sip_uri).await?;
            bind_done_for_ipv6 = true;
        }
        if let Some(sip_uri) = transport_by_ip_type.ipv4 {
            match enable_transport(&sip_uri).await {
                Ok(()) => (),
                Err(_) if bind_done_for_ipv6 => {
                    warn!("Transport {} already bound to IPv6, skipping IPv4", sip_uri);
                }
                Err(err) => return Err(err),
            }
        }
    }

    wait_for_signal().await?;

    info!("Shutting down...");

    Ok(())
}

#[cfg(unix)]
#[tracing::instrument]
async fn wait_for_signal() -> Result<(), std::io::Error> {
    use tokio::signal::unix::{SignalKind, signal};

    let mut pipe_signal_stream = signal(SignalKind::pipe())?;
    let mut int_signal_stream = signal(SignalKind::interrupt())?;
    let mut term_signal_stream = signal(SignalKind::terminate())?;

    loop {
        select! {
            _ = pipe_signal_stream.recv() => (), // Ignore pipe signal
            _ = int_signal_stream.recv() => break,
            _ = term_signal_stream.recv() => break,
        }
    }

    Ok(())
}

#[cfg(windows)]
#[tracing::instrument]
async fn wait_for_signal() -> Result<(), std::io::Error> {
    use tokio::signal::windows::{ctrl_c, ctrl_close, ctrl_logoff, ctrl_shutdown};

    let mut ctrl_c_signal = ctrl_c()?;
    let mut ctrl_close_signal = ctrl_close()?;
    let mut ctrl_logoff_signal = ctrl_logoff()?;
    let mut ctrl_shutdown_signal = ctrl_shutdown()?;

    select! {
        _ = ctrl_c_signal.recv() => (),
        _ = ctrl_close_signal.recv() => (),
        _ = ctrl_logoff_signal.recv() => (),
        _ = ctrl_shutdown_signal.recv() => (),
    }

    Ok(())
}

async fn enable_transport(uri: &SipUri) -> Result<(), std::io::Error> {
    debug!("Enabling transport {}", uri);
    if uri.is_secure() {
        unimplemented!();
    } else {
        let ip = uri.host().ip().unwrap();
        let port = uri.port().unwrap();
        match uri.transport() {
            Some(Transport::Udp) => {
                let _socket = UdpSocket::bind((*ip, port)).await?;
                // TODO
            }
            Some(Transport::Tcp) => {
                let listener = TcpListener::bind((*ip, port)).await?;
                tokio::spawn(async move {
                    loop {
                        let _result = listener.accept().await;
                        // TODO
                    }
                });
            }
            _ => (),
        }
        Ok(())
    }
}
