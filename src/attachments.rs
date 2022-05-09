use std::{
    error::Error,
    io::{self, ErrorKind},
};

use log::debug;
use mime_to_ext::MIME_DATA_MAP;

use crate::{
    types::{Attachment, TelegramMessageData, MyResult},
    utils::make_error,
};

pub fn get_audio_attachments<'a>(
    message_data: &TelegramMessageData<'a>,
) -> Result<Vec<Attachment<'a>>, Box<dyn Error + Sync + Send>> {
    let mut attachments = Vec::new();

    if let Some(audio) = message_data.audio {
        let file_ext = match audio.mime_type.clone() {
            Some(mime) => {
                let mime_type = format!("{}/{}", mime.type_(), mime.subtype());
                let ext = match MIME_DATA_MAP.get(&mime_type) {
                    Some(mime_data) => {
                        debug!(
                            "Found mime type `{}`, writing ext `{}`",
                            mime_type, mime_data.ext
                        );
                        mime_data.ext.as_str()
                    }
                    None => {
                        return Err(Box::new(io::Error::new(
                            ErrorKind::Other,
                            format!("Could not look up mime type `{}`", mime_type),
                        )));
                    }
                };
                ext
            }
            None => {
                return Err(Box::new(io::Error::new(
                    ErrorKind::Other,
                    "Failed to get mime type for audio",
                )));
            }
        };

        let filename = format!("{}{}", audio.file_unique_id, file_ext);

        attachments.push(Attachment::new(
            filename,
            audio.file_id.clone(),
            audio.file_size,
        ));
    }
    Ok(attachments)
}

pub fn get_file_attachments<'a>(
    message_data: &TelegramMessageData<'a>,
) -> Result<Vec<Attachment<'a>>, Box<dyn Error + Sync + Send>> {
    let mut attachments = Vec::new();

    if let Some(file) = message_data.file {
        let file_ext = match file.mime_type.clone() {
            Some(mime) => {
                let mime_type = format!("{}/{}", mime.type_(), mime.subtype());
                let ext = match MIME_DATA_MAP.get(&mime_type) {
                    Some(mime_data) => {
                        debug!(
                            "Found mime type `{}`, writing ext `{}`",
                            mime_type, mime_data.ext
                        );
                        mime_data.ext.as_str()
                    }
                    None => {
                        return Err(Box::new(io::Error::new(
                            ErrorKind::Other,
                            format!("Could not look up ext for file mime type `{}`", mime_type),
                        )));
                    }
                };
                ext
            }
            None => {
                return Err(Box::new(io::Error::new(
                    ErrorKind::Other,
                    "Failed to get mime type for file",
                )));
            }
        };

        let filename = match &file.file_name {
            Some(filename) => filename.to_owned(),
            None => format!("{}{}", file.file_unique_id, file_ext),
        };

        attachments.push(Attachment::new(
            filename,
            file.file_id.clone(),
            file.file_size,
        ));
    }
    Ok(attachments)
}

/// Stickers have no mime type, so we have to guess the file extension I guess
pub fn get_sticker_attachments<'a>(
    message_data: &TelegramMessageData<'a>,
) -> Result<Vec<Attachment<'a>>, Box<dyn Error + Sync + Send>> {
    let mut attachments = Vec::new();

    if let Some(sticker) = message_data.sticker {
        // Try to get the filename
        let filename = if sticker.is_animated {
            return Err(Box::new(io::Error::new(
                ErrorKind::Other,
                "Animated stickers are not supported yet",
            )));
            // format!("{}.tgs", sticker.file_unique_id)
        } else if sticker.is_video {
            format!("{}.webm", sticker.file_unique_id)
        } else {
            format!("{}.png", sticker.file_unique_id)
        };
        attachments.push(Attachment::new(
            filename,
            sticker.file_id.clone(),
            sticker.file_size,
        ));
    }
    Ok(attachments)
}

pub fn get_video_attachments<'a>(
    message_data: &TelegramMessageData<'a>,
) -> MyResult<Vec<Attachment<'a>>> {
    let mut attachments = Vec::new();

    if let Some(video) = message_data.video {
        // Try to get the filename
        let filename = match &video.file_name {
            // Quick return if we have it
            Some(filename) => {
                debug!("Found video filename: {}", filename);
                filename.to_owned()
            }
            None => {
                // Otherwise, do a lott of work to get it
                debug!("No filename found for video, trying to get it");
                let file_ext = match video.mime_type.clone() {
                    Some(mime) => {
                        let mime_type = format!("{}/{}", mime.type_(), mime.subtype());
                        let ext = match MIME_DATA_MAP.get(&mime_type) {
                            Some(mime_data) => mime_data.ext.as_str(),
                            None => {
                                return Err(make_error(&format!(
                                    "Could not look up ext for video mime type `{}`",
                                    mime_type
                                )));
                            }
                        };
                        ext
                    }
                    None => {
                        return Err(Box::new(io::Error::new(
                            ErrorKind::Other,
                            "Failed to get mime type for video",
                        )));
                    }
                };

                format!("{}{}", video.file_unique_id, file_ext)
            }
        };
        attachments.push(Attachment::new(
            filename,
            video.file_id.clone(),
            video.file_size,
        ));
    }
    Ok(attachments)
}

pub fn get_photo_attachments<'a>(
    message_data: &TelegramMessageData<'a>,
) -> Option<Vec<Attachment<'a>>> {
    if let Some(photo) = message_data.photos.and_then(|photos| photos.last()) {
        let mut attachments = Vec::new();

        let filename = format!("{}.jpg", photo.file_unique_id);

        attachments.push(Attachment::new(
            filename,
            photo.file_id.clone(),
            photo.file_size,
        ));

        return Some(attachments);
    }
    None
}
