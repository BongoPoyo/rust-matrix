#![allow(unused_imports)]
#![allow(dead_code)]
use anyhow::Result;
use clap::Parser;
use colored::Colorize;
#[cfg(windows)]
use colored::control;
#[cfg(windows)]
use crossterm;
use keyring::Entry;
use matrix_sdk::{
    AuthSession, Client, SqliteCryptoStore, SqliteEventCacheStore, SqliteStateStore,
    authentication::matrix::MatrixSession,
    config::StoreConfig,
    encryption::{BackupDownloadStrategy, EncryptionSettings},
    reqwest::Url,
    ruma::OwnedRoomId,
};
use matrix_sdk::{
    config::SyncSettings,
    ruma::{UserId, events::room::message::SyncRoomMessageEvent, user_id},
};
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use whoami;

macro_rules! logln {
    ($($arg:tt)*) => {
        println!(
            "{} {}",
            "[Main]".bold().green(),
            format!($($arg)*)
        );
    };
}

#[derive(Debug, Parser)]
struct Cli {
    /// The homeserver the client should connect to.
    server_name: String,

    /// The path where session specific data should be stored.
    #[clap(default_value = "/tmp/")]
    session_path: PathBuf,

    /// Set the proxy that should be used for the connection.
    #[clap(short, long, env = "PROXY")]
    proxy: Option<Url>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    enable_ansi_support();
    let cli = Cli::parse();
    logln!("Building client with https://matrix.org");

    let client = configure_client(cli).await?;
    log_in_or_restore_session(&client, session_path);
    // First we need to log in.

    client.add_event_handler(|ev: SyncRoomMessageEvent| async move {
        logln!("Received a message {:#?}", ev);
    });

    // Syncing is important to synchronize the client state with the server.
    // This method will never return unless there is an error.
    client.sync(SyncSettings::default()).await?;

    Ok(())
}
async fn configure_client(cli: Cli) -> Result<Client> {
    let Cli {
        server_name,
        session_path,
        proxy,
    } = cli;

    let mut client_builder = Client::builder()
        .store_config(
            StoreConfig::new("multiverse".to_owned())
                .crypto_store(SqliteCryptoStore::open(session_path.join("crypto"), None).await?)
                .state_store(SqliteStateStore::open(session_path.join("state"), None).await?)
                .event_cache_store(
                    SqliteEventCacheStore::open(session_path.join("cache"), None).await?,
                ),
        )
        .server_name_or_homeserver_url(&server_name)
        .with_encryption_settings(EncryptionSettings {
            auto_enable_cross_signing: true,
            backup_download_strategy: BackupDownloadStrategy::AfterDecryptionFailure,
            auto_enable_backups: true,
        })
        .with_enable_share_history_on_invite(true);

    if let Some(proxy_url) = proxy {
        client_builder = client_builder.proxy(proxy_url).disable_ssl_verification();
    }

    let client = client_builder.build().await?;

    // Try reading a session, otherwise create a new one.
    log_in_or_restore_session(&client, &session_path).await?;

    Ok(client)
}
async fn log_in_or_restore_session(client: &Client, session_path: &Path) -> Result<()> {
    let session_path = session_path.join("session.json");

    if let Ok(serialized) = std::fs::read_to_string(&session_path) {
        let session: MatrixSession = serde_json::from_str(&serialized)?;
        client.restore_session(session).await?;
    } else {
        login_with_password(client).await?;

        // Immediately save the session to disk.
        if let Some(session) = client.session() {
            let AuthSession::Matrix(session) = session else {
                panic!("unexpected OAuth 2.0 session")
            };
            let serialized = serde_json::to_string(&session)?;
            std::fs::write(session_path, serialized)?;

            println!("saved session");
        }
    }

    Ok(())
}

/// Asks the user of a username and password, and try to login using the matrix
/// auth with those.
async fn login_with_password(client: &Client) -> Result<()> {
    println!("Logging in with username and password…");

    loop {
        print!("\nUsername: ");
        io::stdout().flush().expect("Unable to write to stdout");
        let mut username = String::new();
        io::stdin()
            .read_line(&mut username)
            .expect("Unable to read user input");
        username = username.trim().to_owned();

        let password = rpassword::prompt_password("Password.")?;

        match client
            .matrix_auth()
            .login_username(&username, password.trim())
            .await
        {
            Ok(_) => {
                println!("Logged in as {username}");
                break;
            }
            Err(error) => {
                println!("Error logging in: {error}");
                println!("Please try again\n");
            }
        }
    }

    Ok(())
}
#[cfg(windows)]
fn enable_ansi_support() {
    println!("[Main] Detected Windows.... Enabling ANSI SUPPORT for colors...");
    control::set_virtual_terminal(true).unwrap();
    // crossterm::terminal::enable_virtual_terminal_processing(std::io::stdout()).unwrap();
}

#[cfg(not(windows))]
fn enable_ansi_support() {
    logln!("Detected UNIX BASED OS....");
}
