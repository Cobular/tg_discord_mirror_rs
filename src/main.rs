use once_cell::sync::OnceCell;
use serenity::{http, model::webhook::Webhook};
use std::collections::HashMap;
use std::error::Error;

use teloxide::{dispatching::UpdateFilterExt, prelude::*, types::ChatId};

use crate::attachments::{
    get_audio_attachments, get_file_attachments, get_photo_attachments, get_sticker_attachments,
    get_video_attachments,
};
use crate::types::{TelegramMessageData, UnifiedMessage, Attachment};
use crate::utils::send_all_webhooks;

mod attachments;
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
}

static WEBHOOK: OnceCell<Webhook> = OnceCell::new();

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let webhook = HTTP
        .get_webhook_from_url(
            "https://discordapp.com/api/webhooks/717098180989898560/717098180989898560",
        )
        .await
        .unwrap();

    WEBHOOK.set(webhook).unwrap();

    log::info!("Starting the forwarder bot...");

    let bot = Bot::from_env().auto_send();

    let handler = dptree::entry().branch(Update::filter_channel_post().endpoint(message_handler));

    Dispatcher::builder(bot, handler)
        .build()
        .setup_ctrlc_handler()
        .dispatch()
        .await;
}

/// Parse the text wrote on Telegram and check if that text is a valid command
/// or not, then match the command. If the command is `/start` it writes a
/// markup with the `InlineKeyboardMarkup`.
async fn message_handler(
    m: Message,
    bot: AutoSend<Bot>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Gets the discord webhook data if the chat is one of the tracked channels
    if let Some(_discord_id) = CHANNEL_DATA.get(&m.chat.id) {
        // Pulls required data off the message
        // See this: https://docs.rs/teloxide/latest/teloxide/prelude/struct.Message.html
        let message_data = TelegramMessageData {
            text: m.text(),
            photos: m.photo(),
            audio: m.audio(),
            file: m.document(),
            sticker: m.sticker(),
            video: m.video(),
        };

        let mut message: UnifiedMessage = UnifiedMessage {
            message_data: Vec::new(),
            message_text: None,
        };

        // Generate the photo attachments
        get_photo_attachments(&message_data, &mut message.message_data);

        // Generate the audio attachments
        get_audio_attachments(&message_data, &mut message.message_data)?;

        // Generate the file attachments
        get_file_attachments(&message_data, &mut message.message_data)?;

        // Generate the sticker attachments
        get_sticker_attachments(&message_data, &mut message.message_data)?;

        // Generate the video attachments
        get_video_attachments(&message_data, &mut message.message_data)?;

        // Sort to start with the smallest files first
        message
            .message_data
            .sort_by(|a, b| a.file_size.cmp(&b.file_size));

        // message.message_data.iter_mut().for_each(|message| async {
        // });

        download_all(&mut message.message_data, &bot).await?;

        // Fire the webhook
        send_all_webhooks(message, WEBHOOK.get().unwrap()).await?;

        // Test save all attachments
        // for attachment in &mut message.message_data {
        //     let path = Path::new("test_media").join(&attachment.file_name);
        //     debug!("Saving file {}", path.display());
        //     let mut file = File::create(&path).await?;
        //     let file_size = attachment.file_size;
        //     let data = attachment.download_file(&bot).await?;

        //     if let Some(file_size) = file_size {
        //         debug!(
        //             "Saving attachment: {:?}, downloaded {}/{} bytes",
        //             &path,
        //             data.len(),
        //             file_size
        //         );
        //     } else {
        //         debug!(
        //             "Saving attachment: {:?}, downloaded {} bytes",
        //             &path,
        //             data.len()
        //         );
        //     }

        //     if data.len() == 0 {
        //         return Err(Box::new(io::Error::new(
        //             io::ErrorKind::Other,
        //             format!("File `{}` had no length", path.display()),
        //         )));
        //     }

        //     file.write_all(data).await?;
        // }
    }
    Ok(())
}

async fn download_all<'a>(attachments: &mut [Attachment<'a>], bot: &AutoSend<Bot>) -> Result<(), Box<dyn Error + Send + Sync>> {
    for attachment in attachments {
        attachment.get_file_if_needed(bot).await?;
    }
    Ok(())
}
