use dotenv::dotenv;
use once_cell::sync::OnceCell;
use serenity::http::Http;
use std::{collections::HashMap, env::var};
use teloxide::{dispatching::UpdateFilterExt, prelude::*, types::ChatId};
use tokio::runtime::Runtime;

use crate::{
    telegram_events::message_handler,
    types::{TgChannelData, WebhookData},
    utils::make_webhook,
};

mod attachments;
mod telegram_events;
mod types;
mod utils;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref HTTP: Http = Http::new("e");
    static ref AVATAR_URL: String = var("BOT_ICON").unwrap_or_else(|_| {
        "https://discord.com/assets/1f0bfc0865d324c2587920a7d80c609b.png".to_string()
    });
    static ref USERNAME: String =
        var("BOT_USERNAME").unwrap_or_else(|_| "Telegram Discord Mirror Bot".to_string());
}

static RUNTIME: OnceCell<Runtime> = OnceCell::new();
static BOT: OnceCell<AutoSend<Bot>> = OnceCell::new();
static CHANNEL_DATA_WEBHOOK: OnceCell<HashMap<ChatId, TgChannelData>> = OnceCell::new();

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
    // Ziah Testing
    let webhook_url_1 = var("WEBHOOK_URL_1").unwrap();
    // Nasa Brain
    let webhook_url_2 = var("WEBHOOK_URL_2").unwrap();
    // Ziah's fwd target #1
    let webhook_url_3 = var("WEBHOOK_URL_3").unwrap();
    // Ziah's real channel
    let webhook_url_4 = var("WEBHOOK_URL_4").unwrap();
    // Ziah's fwd target #2
    let webhook_url_5 = var("WEBHOOK_URL_5").unwrap();

    let mut channel_data: HashMap<ChatId, TgChannelData> = HashMap::new();

    // My Channel
    channel_data.insert(
        ChatId(-1001765404638),
        TgChannelData {
            chat_id: ChatId(-1001765404638),
            webhooks: vec![
                WebhookData {
                    raw_webhook: make_webhook(&webhook_url_1).await.unwrap(),
                    webhook_username: "The Queen's Herald".into(),
                    icon_url: "https://cdn.discordapp.com/attachments/849900302965669919/994085960130244639/unknown.png".into()
                },
                WebhookData {
                    raw_webhook: make_webhook(&webhook_url_2).await.unwrap(),
                    webhook_username: "eeee??".into(),
                    icon_url: "https://cdn.discordapp.com/attachments/849900302965669919/994085960130244639/unknown.png".into()
                }
            ],
        },
    );

    // Ziah's channel
    channel_data.insert(
        ChatId(-1001514642130),
        TgChannelData {
            chat_id: ChatId(-1001514642130),
            webhooks: vec![
                WebhookData {
                    raw_webhook: make_webhook(&webhook_url_3).await.unwrap(),
                    webhook_username: "The Queen's Herald".into(),
                    icon_url: "https://cdn.discordapp.com/attachments/849900302965669919/994085960130244639/unknown.png".into()
                },
                WebhookData {
                    raw_webhook: make_webhook(&webhook_url_4).await.unwrap(),
                    webhook_username: "The Queen's Herald".into(),
                    icon_url: "https://cdn.discordapp.com/attachments/849900302965669919/994085960130244639/unknown.png".into()
                },
                WebhookData {
                    raw_webhook: make_webhook(&webhook_url_5).await.unwrap(),
                    webhook_username: "The Queen's Herald".into(),
                    icon_url: "https://cdn.discordapp.com/attachments/849900302965669919/994085960130244639/unknown.png".into()
                },
            ],
        },
    );

    CHANNEL_DATA_WEBHOOK.set(channel_data).unwrap();

    let bot = Bot::from_env().auto_send();

    BOT.set(bot).unwrap();

    let handler = dptree::entry().branch(Update::filter_channel_post().endpoint(message_handler));

    Dispatcher::builder(BOT.get().unwrap(), handler)
        .build()
        .setup_ctrlc_handler()
        .dispatch()
        .await;
}
