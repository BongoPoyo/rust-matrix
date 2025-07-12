#![allow(unused_imports)]
#![allow(dead_code)]
use keyring::Entry;
use matrix_sdk::{
    Client,
    config::SyncSettings,
    ruma::{UserId, events::room::message::SyncRoomMessageEvent, user_id},
};

use whoami;

use colored::Colorize;
#[cfg(windows)]
use colored::control;
#[cfg(windows)]
use crossterm;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::File;
use std::io;
use std::path::Path;
#[derive(Serialize, Deserialize, Debug)]
struct MatrixLoginInfo {
    username: String,
    password: String,
}

macro_rules! logln {
    ($($arg:tt)*) => {
        println!(
            "{} {}",
            "[Main]".bold().green(),
            format!($($arg)*)
        );
    };
}

static SERVICE_NAME: &str = "rust-matrix";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    enable_ansi_support();

    logln!("Building client with https://matrix.org");
    let client = Client::builder()
        .homeserver_url("https://matrix.org")
        .build()
        .await?;

    if let Some(session) = client.session() {
        client.restore_session(session).await?;
        logln!("Logged in with stored session!");
    } else {
        let login_info = get_user_login_info();
        client
            .matrix_auth()
            .login_username(login_info.username.as_str(), login_info.password.as_str())
            .send()
            .await?;
        // fallback to login with stored creds from keyring
    }
    // First we need to log in.

    client.add_event_handler(|ev: SyncRoomMessageEvent| async move {
        logln!("Received a message {:#?}", ev);
    });

    // Syncing is important to synchronize the client state with the server.
    // This method will never return unless there is an error.
    client.sync(SyncSettings::default()).await?;

    Ok(())
}

fn get_user_login_info() -> MatrixLoginInfo {
    let mut matrix_login_info = MatrixLoginInfo {
        username: "".to_string(),
        password: "".to_string(),
    };
    let username = whoami::username();
    logln!("Current user: {}", username);
    let entry = Entry::new(SERVICE_NAME, username.as_str());

    match entry {
        Ok(entry) => {
            logln!("Entry exists: {:?}", entry);
            match entry.get_password() {
                Ok(data) => {
                    logln!("Found info: {}", data);
                }

                Err(e) => {
                    logln!("Error getting password: {}", e);

                    logln!("Enter username: ");
                    let mut username: String = String::new();
                    io::stdin()
                        .read_line(&mut username)
                        .expect("failed to readline");

                    logln!("Enter password: ");
                    let mut password: String = String::new();
                    io::stdin()
                        .read_line(&mut password)
                        .expect("failed to readline");

                    // Remove trailing newlines
                    let username = username.trim().to_string();
                    let password = password.trim().to_string();

                    matrix_login_info = MatrixLoginInfo {
                        username: username,
                        password: password,
                    };

                    let serialized =
                        serde_json::to_string(&matrix_login_info).expect("failed to serialize");

                    if let Err(err) = entry.set_password(&serialized) {
                        logln!("Failed to store credentials: {}", err);
                    } else {
                        logln!("Login info saved to keyring.");
                    }
                }
            }
        }

        Err(e) => {
            logln!("Error getting entry: {}", e);
        }
    }

    matrix_login_info
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
