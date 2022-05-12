use std::{borrow::Cow, error::Error as DynError};

use serenity::{
    http,
    model::{channel::AttachmentType, webhook::Webhook},
};
use teloxide::types::{Animation, Audio, Document, PhotoSize, Sticker, Video};
use tokio::task::{self, JoinHandle};

use crate::utils::download_file;

/// Data comes from https://docs.rs/teloxide/latest/teloxide/types/struct.MessageEntity.html
#[derive(Debug)]
pub struct TelegramMessageData<'a> {
    /// Caption or text of the message
    pub text: Option<&'a str>,
    pub photos: Option<&'a [PhotoSize]>,
    pub audio: Option<&'a Audio>,
    pub file: Option<&'a Document>,
    pub sticker: Option<&'a Sticker>,
    pub video: Option<&'a Video>,
    pub gif: Option<&'a Animation>,
}

pub type InMemoryFile<'a> = Cow<'a, [u8]>;

pub type BoxedError = Box<dyn DynError + Send + Sync>;
pub type MyResult<T> = Result<T, BoxedError>;

pub struct Attachment<'a> {
    pub file_name: String,
    pub file_id: String,
    pub file_data: JoinHandle<MyResult<InMemoryFile<'a>>>,
    pub file_size: Option<u32>,
}

pub struct UnifiedMessage<'a> {
    pub attachments: Vec<Attachment<'a>>,
    pub message_text: Option<String>,
}

pub struct DiscordChannel {
    pub webhook: Webhook,
    pub username: String,
    pub avatar_url: String,
}

impl DiscordChannel {
    pub async fn new(
        webhook_url: &str,
        username: &str,
        avatar_url: &str,
        http: &http::Http,
    ) -> MyResult<Self> {
        let webhook = http.get_webhook_from_url(webhook_url).await?;
        Ok(Self {
            webhook,
            username: username.to_string(),
            avatar_url: avatar_url.to_string(),
        })
    }
}

impl<'a> Attachment<'a> {
    // Create a new file, starting the download but not joining it so we can download while doing other things.
    pub fn new(file_name: String, file_id: String, file_size: Option<u32>) -> Self {
        let cloned_file_id = file_id.clone();
        let future = task::spawn(async move { download_file(cloned_file_id, file_size).await });

        Self {
            file_name,
            file_id,
            file_size,
            file_data: future,
        }
    }

    pub async fn to_discord_attachment(self: Attachment<'a>) -> MyResult<AttachmentType<'a>> {
        Ok(AttachmentType::Bytes {
            filename: self.file_name.clone(),
            data: self.get_file_or_wait().await?,
        })
    }

    pub async fn get_file_or_wait(self) -> MyResult<Cow<'a, [u8]>> {
        let task_result = self.file_data.await;

        match task_result {
            Ok(request_result) => match request_result {
                Ok(file_data) => Ok(file_data),
                Err(err) => Err(err),
            },
            Err(e) => Err(Box::new(e)),
        }
    }
}
