use std::{borrow::Cow, error::Error, io};

use futures::future;
use log::{debug, info, warn};
use serenity::{
    http,
    model::{channel::AttachmentType, webhook::Webhook},
};
use teloxide::{net::Download, prelude::*};

use crate::{
    types::{InMemoryFile, UnifiedMessage},
    BOT,
};

/// Download a file given it's file ID and a bot instance
pub async fn download_file<'a>(
    file_id: String,
    file_size: Option<u32>,
) -> Result<InMemoryFile<'a>, Box<dyn Error + Send + Sync>> {
    // Pull the bot reference
    let bot = BOT
        .get()
        .ok_or_else(|| make_error("Failed to get ref to Bot"))?;

    // Get the file info from telegram
    let tg_file = bot.get_file(file_id).send().await?;
    debug!("File data: {:#?}", tg_file);

    // Create the vec of bytes, either empty or with known size
    let mut im_file = if let Some(size) = file_size {
        Vec::with_capacity(size as usize)
    } else {
        Vec::new()
    };

    // Download the actual file
    bot.download_file(&tg_file.file_path, &mut im_file).await?;
    // Return a cow that owns the data
    Ok(Cow::from(im_file))
}

/// Create and fire off a single webhook
pub async fn send_one_webhook<'a>(
    message_text: Option<String>,
    attachment_slice: Vec<AttachmentType<'a>>,
    webhook: &Webhook,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let http = http::Http::new("e");

    webhook
        .execute(http, true, |hook| {
            let hook = match message_text {
                Some(text) => hook.content(text),
                None => hook,
            };
            for file in attachment_slice {
                hook.add_file(file);
            }
            hook
        })
        .await?;

    Ok(())
}

/// Coordinate sending many webhooks if we need to to fit under the filesize limit
pub async fn send_all_webhooks<'a>(
    attachments: UnifiedMessage<'a>,
    webhook: &Webhook,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Create the vec of pending downloads
    let discord_attachment = attachments
        .attachments
        .into_iter()
        .map(|attachment| attachment.to_discord_attachment());

    // wait for all to finish and discard the mistakes
    let discord_attachments: Vec<_> = future::join_all(discord_attachment)
        .await
        .into_iter()
        .filter_map(|attachment| match attachment {
            Ok(attachment) => Some(attachment),
            Err(_) => {
                warn!("Failed to convert attachment to discord attachment");
                None
            }
        })
        .collect();

    if attachments.message_text.is_some() || !discord_attachments.is_empty() {
        send_one_webhook(attachments.message_text, discord_attachments, webhook).await?;
        info!("Sent webhook");
    } else {
        warn!("No message text or attachments to send, didn't send webhook");
    }
    Ok(())
}

/// Makes a boxed error object with the message
#[inline]
pub fn make_error(message: &str) -> Box<dyn Error + Send + Sync> {
    Box::new(io::Error::new(io::ErrorKind::Other, message.to_string()))
}
