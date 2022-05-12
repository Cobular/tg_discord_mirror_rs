use once_cell::sync::OnceCell;
use serenity::{http, model::webhook::Webhook};
use std::collections::HashMap;
use tokio::runtime::Runtime;
use parking_lot::RwLock;
use teloxide::{dispatching::UpdateFilterExt, prelude::*, types::ChatId};

use crate::{telegram_events::message_handler, types::DiscordChannel};

mod attachments;
mod telegram_events;
mod types;
mod utils;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    /// Stores telegram chat id to discord webhook data mapping
    static ref CHANNEL_DATA: RwLock<HashMap<ChatId, DiscordChannel>> = {
        RwLock::new(HashMap::new())
    };
    static ref HTTP: serenity::http::client::Http = {
        http::Http::new("e")
    };
}

// static WEBHOOK: OnceCell<Webhook> = OnceCell::new();
static RUNTIME: OnceCell<Runtime> = OnceCell::new();
static BOT: OnceCell<AutoSend<Bot>> = OnceCell::new();

fn main() {
    pretty_env_logger::init();
    log::info!("Starting the runtime...");

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    RUNTIME.set(rt).unwrap();

    RUNTIME.get().unwrap().block_on(async_main());
}

async fn async_main() {
    // Need to scope the read write guard here so it drops
    {
        let channel_data = CHANNEL_DATA.write();

    channel_data.insert(
        ChatId(-1), 
        DiscordChannel::new(
            "https://discord.com/api/webhooks/973288685242040351/iuMJRUsKk-j7dmmHLp4B78zSaKYJE6BGudFv9oYBpmI_bcFMLYSryBP4M61LeiszwqHn", 
            "test", 
            "test", 
            &HTTP
        ).await.unwrap()
    );
}

    let bot = Bot::from_env().auto_send();

    BOT.set(bot).unwrap();

    let handler = dptree::entry().branch(Update::filter_channel_post().endpoint(message_handler));

    Dispatcher::builder(BOT.get().unwrap(), handler)
        .build()
        .setup_ctrlc_handler()
        .dispatch()
        .await;
}
