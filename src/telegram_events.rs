use std::borrow::Borrow;
use std::io;
use std::path::Path;

use log::{debug, warn};

use teloxide::types::Message;

use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::attachments::{
    get_audio_attachments, get_file_attachments, get_gif_attachments, get_photo_attachments,
    get_sticker_attachments, get_video_attachments,
};
use crate::types::{MyResult, TelegramMessageData, UnifiedMessage};
use crate::utils::send_all_webhooks;
use crate::{CHANNEL_DATA};

/// Parse the text wrote on Telegram and check if that text is a valid command
/// or not, then match the command. If the command is `/start` it writes a
/// markup with the `InlineKeyboardMarkup`.
pub async fn message_handler(m: Message) -> MyResult<()> {
    // Gets the discord webhook data if the chat is one of the tracked channels
    if let Some(discord_channel) = CHANNEL_DATA.read().get(&m.chat.id) {
        // Pulls required data off the message
        // See this: https://docs.rs/teloxide/latest/teloxide/prelude/struct.Message.html
        let text = if m.text().is_none() {
            m.caption()
        } else {
            m.text()
        };
        let message_data = TelegramMessageData {
            text,
            photos: m.photo(),
            audio: m.audio(),
            file: m.document(),
            sticker: m.sticker(),
            video: m.video(),
            gif: m.animation(),
        };

        let mut message: UnifiedMessage = UnifiedMessage {
            attachments: Vec::new(),
            message_text: text.map(|s| s.to_string()),
        };

        // Generate the photo attachments
        if let Some(photo_attachment) = get_photo_attachments(&message_data) {
            message.attachments.extend(photo_attachment.into_iter());
        }

        // Generate the audio attachments
        match get_audio_attachments(&message_data) {
            Ok(audio_attachments) => message.attachments.extend(audio_attachments.into_iter()),
            Err(_) => warn!("Failed to parse audio attachments"),
        };
        // Generate the file attachments
        match get_file_attachments(&message_data) {
            Ok(file_attachments) => message.attachments.extend(file_attachments.into_iter()),
            Err(_) => warn!("Failed to parse file attachments"),
        };

        // Generate the sticker attachments
        match get_sticker_attachments(&message_data) {
            Ok(sticker_attachments) => message.attachments.extend(sticker_attachments.into_iter()),
            Err(_) => warn!("Failed to parse sticker attachments"),
        };

        // Generate the video attachments
        match get_video_attachments(&message_data) {
            Ok(video_attachments) => message.attachments.extend(video_attachments.into_iter()),
            Err(_) => warn!("Failed to parse video attachments"),
        };

        // Generate the sticker attachments
        match get_gif_attachments(&message_data) {
            Ok(gif_attachments) => message.attachments.extend(gif_attachments.into_iter()),
            Err(_) => warn!("Failed to parse gif attachments"),
        };

        // Sort to start with the smallest files first
        message
            .attachments
            .sort_by(|a, b| a.file_size.cmp(&b.file_size));

        // Test save all attachments
        // _download_attachments(&message).await?;

        // Fire the webhooks
        send_all_webhooks(message, &discord_channel.webhook).await?;
    }
    Ok(())
}

async fn _download_attachments(message: UnifiedMessage<'_>) -> MyResult<()> {
    for attachment in message.attachments {
        let path = Path::new("test_media").join(&attachment.file_name);
        debug!("Saving file {}", path.display());
        let mut file = File::create(&path).await?;
        let file_size = attachment.file_size;
        let data = attachment.get_file_or_wait().await?;

        if let Some(file_size) = file_size {
            debug!(
                "Saving attachment: {:?}, downloaded {}/{} bytes",
                &path,
                data.len(),
                file_size
            );
        } else {
            debug!(
                "Saving attachment: {:?}, downloaded {} bytes",
                &path,
                data.len()
            );
        }

        if data.len() == 0 {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                format!("File `{}` had no length", path.display()),
            )));
        }

        file.write_all(data.borrow()).await?;
    }
    Ok(())
}
