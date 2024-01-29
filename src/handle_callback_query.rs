use reqwest::Url;
use teloxide::{prelude::*};
use teloxide::adaptors::DefaultParseMode;
use teloxide::types::{InputFile, KeyboardButton, KeyboardMarkup};

use crate::bot::*;
use crate::db::DatabaseHandler;

pub type DLEBot = DefaultParseMode<Bot>;


pub async fn handle_callback_query(
    db: DatabaseHandler,
    bot: DLEBot,
    query: CallbackQuery,
) -> ResponseResult<()> {
    match query.data.as_deref() {
        Some(CITY_CALLBACK_MSK) => {
            select_city(db, &bot, query, MOSCOW_CITY).await?;
        }
        Some(CITY_CALLBACK_IZH) => {
            select_city(db, &bot, query, IZHEVSK_CITY).await?;
        }
        Some(CHANGE_WISH_CALLBACK) => {
            let user = db.get_user(query.from.id.0 as i64).await.unwrap();
            let message = query.message.unwrap();
            if user.santa.is_some() {
                bot.edit_message_text(message.chat.id, message.id, "Поздно пить Боржоми! После распределения ролей нельзя менять список пожеланий, можешь написать в чате своему санте").await?;
            } else {
                bot.edit_message_text(message.chat.id, message.id, "Введите список желаний, я его передам Санте").await?;
            }
        }
        _ => {
            log::warn!("Unrecognized callback query: {:?}", &query);
            bot.send_message(query.message.unwrap().chat.id, "Выберите город из меню.")
                .await?;
        }
    }

    Ok(())
}

async fn select_city(db: DatabaseHandler, bot: &DLEBot, query: CallbackQuery, city: &str) -> ResponseResult<()> {
    let message = query.message.unwrap();
    let text = format!(include_str!("templates/state_4_wait_notify.txt"), city);
    bot.edit_message_text(message.chat.id, message.id, text).await?;
    let keyboard = KeyboardMarkup::new([
        [KeyboardButton::new(CHANGE_WISH_LIST)]
    ]).resize_keyboard(true);

    let url_state_1 = "https://i.pinimg.com/originals/23/c7/74/23c774529515a93be6485e75faeeed36.gif";
    bot.send_animation(message.chat.id, InputFile::url(Url::parse(url_state_1).unwrap()))
        .disable_notification(true)
        .await?;
    bot.send_message(message.chat.id, "Теперь дождись, когда остальные внучата запишутся и мы торжественно распределим Тайных Сант и подопечных.")
        .reply_markup(keyboard)
        .await?;
    bot.send_message(message.chat.id, "Но ни в коем случае не раскрывай кто ты, иначе дух праздника и волшебства пропадет навсегда!").await?;
    let url_state_1 = "https://i.pinimg.com/originals/60/d3/03/60d303c9d5ed80378dfcbcc1923e8acc.gif";
    bot.send_animation(message.chat.id, InputFile::url(Url::parse(url_state_1).unwrap()))
        .disable_notification(true)
        .await?;
    let mut user = db.get_user(query.from.id.0 as i64).await.unwrap();
    user.city = city.to_string();
    user.state = None;
    db.save_user(user).await;
    Ok(())
}
