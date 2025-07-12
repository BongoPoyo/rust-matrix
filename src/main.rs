#![allow(unused_imports)]
#![allow(dead_code)]
use keyring::Entry;
use matrix_sdk::{
    Client,
    config::SyncSettings,
    ruma::{UserId, events::room::message::SyncRoomMessageEvent, user_id},
};

use whoami;

use std::fs::File;
use std::path::Path;

use colored::Colorize;
#[cfg(windows)]
use colored::control;
#[cfg(windows)]
use crossterm;

struct MatrixLoginInfo {
    username: &'static str,
    password: &'static str,
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

    let login_info = get_user_login_info();
    let user_id = user_id!("@bongopoyo:matrix.org");
    let client = Client::builder()
        .server_name(user_id.server_name())
        .build()
        .await?;

    // First we need to log in.
    client
        .matrix_auth()
        .login_username(user_id, login_info.password)
        .send()
        .await?;

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
        username: "",
        password: "",
    };
    let username = whoami::username();
    logln!("Current user: {}", username);
    let entry = Entry::new(SERVICE_NAME, username.as_str());

    match entry {
        Ok(entry) => {
            logln!("Entry exists: {:?}", entry);
            match entry.get_password() {
                Ok(password) => {
                    logln!("Password: {}", password);
                }
                Err(e) => {
                    logln!("Error getting password: {}", e);
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
