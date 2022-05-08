use std::{
    borrow::{Cow, Borrow},
    error::Error as DynError,
    io::{Error, ErrorKind},
};

use serenity::model::channel::AttachmentType;
use teloxide::{
    adaptors::AutoSend,
    types::{Audio, Document, PhotoSize, Sticker, Video},
    Bot,
};

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
}

pub type InMemoryFile<'a> = Cow<'a, [u8]>;

pub struct Attachment<'a> {
    pub file_name: String,
    pub file_id: String,
    _file_data: InMemoryFile<'a>,
    pub file_size: Option<u32>,
}

pub struct UnifiedMessage<'a> {
    pub message_data: Vec<Attachment<'a>>,
    pub message_text: Option<String>
}

impl<'a> TryInto<AttachmentType<'a>> for Attachment<'a> {
    type Error = Error;

    fn try_into(self: Attachment<'a>) -> Result<AttachmentType<'a>, Error> {
        if self._file_data.len() != 0 {
            Ok(AttachmentType::Bytes {
                data: self._file_data,
                filename: self.file_name,
            })
        } else {
            Err(Error::new(ErrorKind::Other, "No file data"))
        }
    }
}

impl<'a> Attachment<'a> {
    pub fn new(file_name: String, file_id: String, file_size: Option<u32>) -> Self {
        Self {
            file_name,
            file_id,
            _file_data: Cow::from(Vec::new()),
            file_size
        }
    }

    pub async fn download_file(
        &'a mut self,
        bot: &AutoSend<Bot>,
    ) -> Result<&'a InMemoryFile<'a>, Box<dyn DynError + Send + Sync>> {
        if self._file_data.len() == 0 {
            let file: Cow<'a, [u8]> = download_file(&self.file_id, bot).await?;
            self._file_data = file;
        };

        return Ok(self._file_data.borrow());
    }
}
