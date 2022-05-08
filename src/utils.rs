use std::{borrow::Cow, error::Error};

use log::debug;
use serenity::{http, model::{webhook::Webhook, channel::AttachmentType}};
use teloxide::{net::Download, prelude::*, Bot};

use crate::types::{Attachment, InMemoryFile, UnifiedMessage};

/// Download a file given it's file ID and a bot instance
pub async fn download_file<'a>(
    file_id: &'a str,
    bot: &AutoSend<Bot>,
) -> Result<InMemoryFile<'a>, Box<dyn Error + Send + Sync>> {
    let tg_file = bot.get_file(file_id).send().await?;
    debug!("File data: {:#?}", tg_file);
    let mut im_file = Vec::new();
    bot.download_file(&tg_file.file_path, &mut im_file).await?;
    Ok(Cow::from(im_file))
}

pub async fn send_one_webhook<'a>(
    message_text: Option<String>,
    attachment_slice: &[AttachmentType<'a>],
    webhook: &Webhook,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let http = http::Http::new("e");

    webhook
        .execute(&http, true, |hook| {
            let mut hook = match message_text {
                Some(text) => hook.content(text),
                None => hook,
            };
            hook.add_files(attachment_slice.into_iter())
        })
        .await?;

    Ok(())
}

pub async fn send_all_webhooks<'a>(attachments: UnifiedMessage<'a>, webhook: &Webhook) {
    let discord_attachments: Vec<AttachmentType> = attachments.message_data.iter().filter_map(|attachment| {
        attachment.try_into().ok()
    }).collect();
}
