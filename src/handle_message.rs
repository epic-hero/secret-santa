use chrono::Utc;
use reqwest::Url;
use sea_orm::prelude::DateTimeWithTimeZone;
use teloxide::{prelude::*};
use teloxide::adaptors::DefaultParseMode;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, InputFile, KeyboardButton, KeyboardMarkup, Me};
use teloxide::utils::command::BotCommands;

use crate::bot::*;
use crate::db::DatabaseHandler;
use crate::types;
use crate::types::User;

pub type DLEBot = DefaultParseMode<Bot>;


pub async fn handle_message(db: DatabaseHandler, bot: DLEBot, msg: Message, me: Me) -> ResponseResult<()> {
    let my_bot = MyBot::new().await;

    let user_id = msg.chat.id.0;
    if let None = msg.text() {
        bot.send_message(msg.chat.id, "Моя твоя не понимать").await?;
        return Ok(());
    }

    let text = msg.text().unwrap();
    let name = msg.chat.username()
        .or(msg.chat.first_name()).unwrap_or(user_id.to_string().as_str())
        .to_string();
    let mut user = db.get_user(user_id).await
        .or(User::default_user(user_id, name)).unwrap();

    match BotCommands::parse(text, me.username()) {
        Ok(Command::Start) => {
            if user.wish_text != "" {
                bot.send_message(msg.chat.id, "Хитрец! Больше одного подарка не положено").await?;
                return Ok(());
            }
            user.state = Option::from(State::ReceiveName);
            db.save_user(user).await;

            bot.send_message(msg.chat.id, "Хо-хо-хо! Приветствую тебя в тайном чате Бай Семьи!").await?;
            let url_state_1 = "https://media.baamboozle.com/uploads/images/153804/1608041764_533336";
            bot.send_animation(msg.chat.id, InputFile::url(Url::parse(url_state_1).unwrap()))
                .disable_notification(true)
                .await?;
            bot.send_message(msg.chat.id, "А почему он тайный, спросишь ты? Потому что здесь мы распределяем Тайных Сант на самый волшебный праздник – Новый Год 🎅🎄 ").await?;

            my_bot.send_start(bot, msg).await?;
            return Ok(());
        }
        Ok(Command::List) => {
            if user_id == ADMIN_ID {
                my_bot.send_list_users(&bot, &msg, &db).await?;
            }
            return Ok(());
        }
        Ok(Command::Notify) => {
            if user_id == ADMIN_ID {
                my_bot.notify(&bot, &db).await?;
            }
            return Ok(());
        }
        Ok(Command::Distribute) => {
            if user_id == ADMIN_ID {
                my_bot.distribute(&bot, &msg, &db).await?;
            }
            return Ok(());
        }
        Ok(Command::Help) => {
            my_bot.send_help(bot, msg).await?;
            return Ok(());
        }
        Err(_) => match text {
            KEY_CHILD_CHAT => {
                let message = match db.find_message(user.chat_id, user.child.unwrap()).await {
                    Some(message_db) => message_db.message
                        .replace(SANTA_PATTERN, "Вы")
                        .replace(CHILD_PATTERN, "Подопечный"),
                    None => String::from("У вас еще нет сообщений с подопечным, но все что ты напишешь ниже я ему покажу")
                };

                let keyboard = KeyboardMarkup::new(
                    [[KeyboardButton::new(KEY_CHILD_CHAT_CLOSE)]])
                    .resize_keyboard(true);

                bot.send_message(msg.chat.id, message)
                    .reply_markup(keyboard)
                    .await?;

                let text = format!("<i>Все что напишете ниже я отправлю Подопечному, чтобы выйти из беседы, нажми '{}</i>'", KEY_CHILD_CHAT_CLOSE);
                bot.send_message(msg.chat.id, text).await?;

                user.state = Option::from(State::ChildChat);
                db.save_user(user).await;
                return Ok(());
            }
            KEY_SANTA_CHAT => {
                let message = match db.find_message(user.santa.unwrap(), user.chat_id).await {
                    Some(message_db) => message_db.message
                        .replace(SANTA_PATTERN, "Санта")
                        .replace(CHILD_PATTERN, "Вы"),
                    None => String::from("У вас еще нет сообщений с сантой, но все что ты напишешь ниже я ему покажу")
                };

                let keyboard = KeyboardMarkup::new(
                    [[KeyboardButton::new(KEY_SANTA_CHAT_CLOSE)]])
                    .resize_keyboard(true);


                bot.send_message(msg.chat.id, message)
                    .reply_markup(keyboard)
                    .await?;
                let text = format!("<i>Все что напишете ниже я отправлю Cанте, чтобы выйти из беседы, нажми '{}</i>'", KEY_SANTA_CHAT_CLOSE);
                bot.send_message(msg.chat.id, text).await?;

                user.state = Option::from(State::SantaChat);
                db.save_user(user).await;
                return Ok(());
            }
            KEY_CHILD_CHAT_CLOSE | KEY_SANTA_CHAT_CLOSE => {
                let keyboard = KeyboardMarkup::new([
                    [KeyboardButton::new(KEY_CHILD_CHAT)],
                    [KeyboardButton::new(KEY_SANTA_CHAT)],
                ]).resize_keyboard(true);

                bot.send_message(msg.chat.id, "Можешь написать подопечнму или Санте:")
                    .reply_markup(keyboard)
                    .await?;
                user.state = None;
                db.save_user(user).await;
                return Ok(());
            }
            CHANGE_WISH_LIST => {
                let inline_keyboard = InlineKeyboardMarkup::new([
                    [InlineKeyboardButton::callback("Изменить", CHANGE_WISH_CALLBACK)],
                ]);

                let text = format!("Твой список желаний:\n{}\nХочешь его изменить?", user.wish_text);
                bot.send_message(msg.chat.id, text)
                    .reply_markup(inline_keyboard)
                    .await?;

                user.state = Option::from(State::ChangeWishList);
                db.save_user(user).await;
                return Ok(());
            }
            _ => {}
        }
    };
    match user.state {
        Some(State::ReceiveName) => {
            match msg.text() {
                Some(username) => {
                    user.username = username.parse().unwrap();
                    user.state = Option::from(State::ReceiveWish);

                    bot.send_message(msg.chat.id, include_str!("templates/state_2_write_name_0.txt")).await?;

                    let url_state_2 = "https://www.sunhome.ru/i/cards/198/elka-animacionnaya-otkritka.orig.gif";
                    bot.send_animation(msg.chat.id, InputFile::url(Url::parse(url_state_2).unwrap()))
                        .disable_notification(true)
                        .await?;

                    bot.send_message(msg.chat.id, include_str!("templates/state_2_write_name_1.txt")).await?;
                    db.save_user(user).await;
                }
                None => {
                    bot.send_message(msg.chat.id, "Отправьте мне обычный текст.").await?;
                }
            }
            return Ok(());
        }
        Some(State::ReceiveWish) => {
            match msg.text() {
                Some(wish) => {
                    let inline_keyboard = InlineKeyboardMarkup::new([
                        [InlineKeyboardButton::callback(MOSCOW_CITY, CITY_CALLBACK_MSK)],
                        [InlineKeyboardButton::callback(IZHEVSK_CITY, CITY_CALLBACK_IZH)]
                    ]);

                    bot.send_message(msg.chat.id, include_str!("templates/stat_3_select_city_0.txt"))
                        .disable_web_page_preview(true)
                        .reply_markup(inline_keyboard)
                        .await?;

                    user.wish_text = wish.parse().unwrap();
                    user.state = Option::from(State::ReceiveCity);
                    db.save_user(user).await;
                }
                None => {
                    bot.send_message(msg.chat.id, "Отправьте мне обычный текст.")
                        .await?;
                }
            }
            return Ok(());
        }
        Some(State::SantaChat) => {
            match msg.text() {
                Some(message_text) => {
                    let santa_id = user.santa.unwrap();
                    let message = types::Message {
                        child_id: user.id,
                        santa_id,
                        message: format!("<b>$child: </b>\n{}", message_text),
                        create_date: DateTimeWithTimeZone::from(Utc::now()),
                    };
                    db.save_message(message).await;

                    bot.send_message(ChatId(santa_id), format!("У вас новое сообщение от подопечного:\n{}", message_text))
                        .await?;
                }
                None => {}
            }
            return Ok(());
        }
        Some(State::ChildChat) => {
            match msg.text() {
                Some(message_text) => {
                    let child_id = user.child.unwrap();
                    let message = types::Message {
                        santa_id: user.id,
                        child_id,
                        message: format!("<b>$santa: </b>\n{}", message_text),
                        create_date: DateTimeWithTimeZone::from(Utc::now()),
                    };
                    db.save_message(message).await;
                    bot.send_message(ChatId(child_id), format!("У вас новое сообщение от Санты:\n{}", message_text))
                        .await?;
                }
                None => {}
            }
            return Ok(());
        }
        Some(State::ChangeWishList) => {
            match msg.text() {
                Some(message_text) => {
                    bot.send_message(msg.chat.id, "Список желаний успешно изменен!").await?;
                    user.wish_text = message_text.to_string();
                    user.state = None;
                    db.save_user(user).await;
                }
                None => {}
            }
            return Ok(());
        }
        _ => {}
    }
    Ok(())
}
