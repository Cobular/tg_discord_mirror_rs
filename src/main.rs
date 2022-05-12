use once_cell::sync::OnceCell;
use serenity::{http, model::webhook::Webhook};
use std::{collections::HashMap, env::var};
use tokio::runtime::Runtime;
use dotenv::dotenv;
use teloxide::{dispatching::UpdateFilterExt, prelude::*, types::ChatId};

use crate::telegram_events::message_handler;

mod attachments;
mod telegram_events;
mod types;
mod utils;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    /// Stores telegram chat id to discord webhook data mapping
    static ref CHANNEL_DATA: HashMap<ChatId, u64> = {
        let mut m = HashMap::new();
        m.insert(ChatId(-1001765404638), 111);
        m
    };
    static ref HTTP: serenity::http::client::Http = {
        http::Http::new("e")
    };
    static ref AVATAR_URL: String = var("BOT_ICON").unwrap_or_else(|_| "https://discord.com/assets/1f0bfc0865d324c2587920a7d80c609b.png".to_string());
    static ref USERNAME: String = var("BOT_USERNAME").unwrap_or_else(|_| "Telegram Discord Mirror Bot".to_string());
}

static WEBHOOK: OnceCell<Webhook> = OnceCell::new();
static RUNTIME: OnceCell<Runtime> = OnceCell::new();
static BOT: OnceCell<AutoSend<Bot>> = OnceCell::new();

fn main() {
    dotenv().ok();
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
    let webhook_url = var("WEBHOOK_URL").unwrap();

    let webhook = HTTP
        .get_webhook_from_url(
            webhook_url.as_str(),
        )
        .await
        .unwrap();

    WEBHOOK.set(webhook).unwrap();

    let bot = Bot::from_env().auto_send();

    BOT.set(bot).unwrap();

    let handler = dptree::entry().branch(Update::filter_channel_post().endpoint(message_handler));

    Dispatcher::builder(BOT.get().unwrap(), handler)
        .build()
        .setup_ctrlc_handler()
        .dispatch()
        .await;
}
