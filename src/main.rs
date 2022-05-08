use log::{debug, info};
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use teloxide::{dispatching::UpdateFilterExt, prelude::*, types::ChatId};
use tokio::{fs::File, io::AsyncWriteExt};

use crate::attachments::{get_audio_attachments, get_file_attachments, get_photo_attachments, get_video_attachments};
use crate::types::{Attachment, MessageData};

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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();
    log::info!("Starting the forwarder bot...");

    let bot = Bot::from_env().auto_send();

    let handler = dptree::entry().branch(Update::filter_channel_post().endpoint(message_handler));

    Dispatcher::builder(bot, handler)
        .build()
        .setup_ctrlc_handler()
        .dispatch()
        .await;
    Ok(())
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
        let message_data = MessageData {
            text: m.text(),
            photos: m.photo(),
            audio: m.audio(),
            file: m.document(),
            sticker: m.sticker(),
            video: m.video(),
        };

        let mut attachments: Vec<Attachment> = Vec::new();

        // Generate the photo attachments
        get_photo_attachments(&message_data, &mut attachments);

        // Generate the audio attachments
        get_audio_attachments(&message_data, &mut attachments)?;

        // Generate the file attachments
        get_file_attachments(&message_data, &mut attachments)?;

        // Generate the sticker attachments

        // Generate the video attachments
        get_video_attachments(&message_data, &mut attachments)?;

        // Test save all attachments
        for attachment in &mut attachments {
            info!("Saving attachment: {:?}", &attachment.file_name);
            let mut file =
                File::create(Path::new("./text_media").join(&attachment.file_name)).await?;
            let file_name = attachment.file_name.clone();
            let file_size = attachment.file_size;
            let data = attachment.download_file(&bot).await?;

            if let Some(file_size) = file_size {
                debug!(
                    "Saving attachment: {:?}, downloaded {}/{} bytes",
                    file_name,
                    data.len(),
                    file_size
                );
            } else {
                debug!(
                    "Saving attachment: {:?}, downloaded {} bytes",
                    file_name,
                    data.len()
                );
            }
            file.write(data).await?;
        }
    }
    Ok(())
}
