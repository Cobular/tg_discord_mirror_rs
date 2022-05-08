use std::{error::Error, borrow::Cow};

use log::debug;
use teloxide::{Bot, prelude::*, net::Download};

use crate::types::InMemoryFile;


/// Download a file given it's file ID and a bot instance
pub async fn download_file<'a>(file_id: &'a str, bot: &AutoSend<Bot>) -> Result<InMemoryFile<'a>, Box<dyn Error + Send + Sync>> {
    let tg_file = bot.get_file(file_id).send().await?;
    debug!("File data: {:#?}", tg_file);
    let mut im_file = Vec::new();
    bot.download_file(&tg_file.file_path, &mut im_file).await?;
    Ok(Cow::from(im_file))
}
