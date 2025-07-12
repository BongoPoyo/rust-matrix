use keyring::Entry;
use matrix_sdk::{
    Client,
    config::SyncSettings,
    ruma::{events::room::message::SyncRoomMessageEvent, user_id},
};

struct MatrixLoginInfo {
    user_id :,
    password : str
}



#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let service = "rust-matrix";

    let user_id = user_id!("@bongopoyo:matrix.org");
    let client = Client::builder()
        .server_name(user_id.server_name())
        .build()
        .await?;

    // First we need to log in.
    client
        .matrix_auth()
        .login_username(user_id, "2zgIThXpip6WX4")
        .send()
        .await?;

    client.add_event_handler(|ev: SyncRoomMessageEvent| async move {
        println!("Received a message {:?}", ev);
    });

    // Syncing is important to synchronize the client state with the server.
    // This method will never return unless there is an error.
    client.sync(SyncSettings::default()).await?;

    Ok(())
}


fn get_user_info() {
}
