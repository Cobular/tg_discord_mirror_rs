use std::{borrow::Cow, error::Error};

use log::debug;
use serenity::{
    http,
    model::{channel::AttachmentType, webhook::Webhook},
};
use teloxide::{net::Download, prelude::*, Bot};

use crate::{
    types::{InMemoryFile, UnifiedMessage},
    HTTP,
};

/// Download a file given it's file ID and a bot instance
pub async fn download_file<'a>(
    file_id: &'a str,
    file_size: Option<u32>,
    bot: &AutoSend<Bot>,
) -> Result<InMemoryFile<'a>, Box<dyn Error + Send + Sync>> {
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

pub async fn send_all_webhooks<'a>(
    attachments: UnifiedMessage<'a>,
    webhook: &Webhook,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let discord_attachments = attachments
        .message_data
        .into_iter()
        .map(|attachment| attachment.into())
        .collect::<Vec<AttachmentType>>();

    send_one_webhook(attachments.message_text, discord_attachments, webhook).await?;
    Ok(())
}
