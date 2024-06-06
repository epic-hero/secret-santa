use async_trait::async_trait;
use chrono::Utc;
use reqwest::Url;
use sea_orm::prelude::DateTimeWithTimeZone;
use teloxide::prelude::Message;
use teloxide::prelude::*;
use teloxide::types::{
    InlineKeyboardButton, InlineKeyboardMarkup, InputFile, KeyboardButton, KeyboardMarkup,
};

use crate::bot::{
    State, CHANGE_WISH_CALLBACK, CHANGE_WISH_LIST, CHILD_PATTERN, CITY_CALLBACK_IZH,
    CITY_CALLBACK_MSK, IZHEVSK_CITY, KEY_CHILD_CHAT, KEY_CHILD_CHAT_CLOSE, KEY_SANTA_CHAT,
    KEY_SANTA_CHAT_CLOSE, MOSCOW_CITY, SANTA_PATTERN,
};
use crate::db::DatabaseHandler;
use crate::types::User;
use crate::{types, SantaBot};

pub(crate) struct ReceiveNameStrategy;

pub(crate) struct ReceiveWishStrategy;

pub(crate) struct SantaChatStrategy;

pub(crate) struct ChildChatStrategy;

pub(crate) struct ChangeWishListStrategy;

pub(crate) struct FinishStrategy;

pub(crate) struct DistributedStrategy;

pub fn state_factory(state: &Option<State>) -> Box<dyn StateStrategy> {
    match state {
        Some(State::ReceiveName) => Box::new(ReceiveNameStrategy),
        Some(State::ReceiveWish) => Box::new(ReceiveWishStrategy),
        Some(State::SantaChat) => Box::new(SantaChatStrategy),
        Some(State::ChildChat) => Box::new(ChildChatStrategy),
        Some(State::ChangeWishList) => Box::new(ChangeWishListStrategy),
        Some(State::Finish) => Box::new(FinishStrategy),
        Some(State::Distributed) => Box::new(DistributedStrategy),
        _ => panic!("State not found"),
    }
}

#[async_trait]
pub(crate) trait StateStrategy: Sync + Send {
    async fn handle(&self, user: User, msg: Message, bot: SantaBot, db: DatabaseHandler);
}

#[async_trait]
impl StateStrategy for DistributedStrategy {
    async fn handle(&self, mut user: User, msg: Message, bot: SantaBot, db: DatabaseHandler) {
        match msg.text() {
            Some(KEY_CHILD_CHAT) => {
                let message = match db.find_message(user.chat_id, user.child.unwrap()).await {
                    Some(message_db) => message_db.message
                        .replace(SANTA_PATTERN, "Вы")
                        .replace(CHILD_PATTERN, "Подопечный"),
                    None => String::from("У вас еще нет сообщений с подопечным, но все что ты напишешь ниже я ему покажу")
                };

                let keyboard = KeyboardMarkup::new([[KeyboardButton::new(KEY_CHILD_CHAT_CLOSE)]])
                    .resize_keyboard(true);

                bot.send_message(msg.chat.id, message)
                    .reply_markup(keyboard)
                    .await
                    .unwrap();

                let text = format!("<i>Все что напишете ниже я отправлю Подопечному, чтобы выйти из беседы, нажми '{}</i>'", KEY_CHILD_CHAT_CLOSE);
                bot.send_message(msg.chat.id, text).await.unwrap();

                user.state = Option::from(State::ChildChat);
                db.save_user(user).await;
            }
            Some(KEY_SANTA_CHAT) => {
                let message = match db.find_message(user.santa.unwrap(), user.chat_id).await {
                    Some(message_db) => message_db.message
                        .replace(SANTA_PATTERN, "Санта")
                        .replace(CHILD_PATTERN, "Вы"),
                    None => String::from("У вас еще нет сообщений с сантой, но все что ты напишешь ниже я ему покажу")
                };

                let keyboard = KeyboardMarkup::new([[KeyboardButton::new(KEY_SANTA_CHAT_CLOSE)]])
                    .resize_keyboard(true);

                bot.send_message(msg.chat.id, message)
                    .reply_markup(keyboard)
                    .await
                    .unwrap();
                let text = format!("<i>Все что напишете ниже я отправлю Cанте, чтобы выйти из беседы, нажми '{}</i>'", KEY_SANTA_CHAT_CLOSE);
                bot.send_message(msg.chat.id, text).await.unwrap();

                user.state = Option::from(State::SantaChat);
                db.save_user(user).await;
            }
            _ => {}
        }
    }
}

#[async_trait]
impl StateStrategy for FinishStrategy {
    async fn handle(&self, mut user: User, msg: Message, bot: SantaBot, db: DatabaseHandler) {
        match msg.text() {
            Some(CHANGE_WISH_LIST) => {
                let inline_keyboard =
                    InlineKeyboardMarkup::new([[InlineKeyboardButton::callback(
                        "Изменить",
                        CHANGE_WISH_CALLBACK,
                    )]]);

                let text = format!(
                    "Твой список желаний:\n{}\nХочешь его изменить?",
                    user.wish_text
                );
                bot.send_message(msg.chat.id, text)
                    .reply_markup(inline_keyboard)
                    .await
                    .unwrap();

                user.state = Option::from(State::ChangeWishList);
                db.save_user(user).await;
            }
            _ => {}
        }
    }
}

#[async_trait]
impl StateStrategy for ChangeWishListStrategy {
    async fn handle(&self, mut user: User, msg: Message, bot: SantaBot, db: DatabaseHandler) {
        match msg.text() {
            Some(message_text) => {
                bot.send_message(msg.chat.id, "Список желаний успешно изменен!")
                    .await
                    .unwrap();
                user.wish_text = message_text.to_string();
                user.state = Option::from(State::Finish);
                db.save_user(user).await;
            }
            None => {}
        }
    }
}

#[async_trait]
impl StateStrategy for ChildChatStrategy {
    async fn handle(&self, mut user: User, msg: Message, bot: SantaBot, db: DatabaseHandler) {
        match msg.text() {
            Some(KEY_CHILD_CHAT_CLOSE) => {
                let keyboard = KeyboardMarkup::new([
                    [KeyboardButton::new(KEY_CHILD_CHAT)],
                    [KeyboardButton::new(KEY_SANTA_CHAT)],
                ])
                .resize_keyboard(true);

                bot.send_message(msg.chat.id, "Можешь написать подопечнму или Санте:")
                    .reply_markup(keyboard)
                    .await
                    .unwrap();
                user.state = Option::from(State::Distributed);
                db.save_user(user).await;
            }
            Some(message_text) => {
                let child_id = user.child.unwrap();
                let message = types::Message {
                    santa_id: user.id,
                    child_id,
                    message: format!("<b>$santa: </b>\n{}", message_text),
                    create_date: DateTimeWithTimeZone::from(Utc::now()),
                };
                db.save_message(message).await;
                bot.send_message(
                    ChatId(child_id),
                    format!("У вас новое сообщение от Санты:\n{}", message_text),
                )
                .await
                .unwrap();
            }
            None => {}
        }
    }
}

#[async_trait]
impl StateStrategy for SantaChatStrategy {
    async fn handle(&self, mut user: User, msg: Message, bot: SantaBot, db: DatabaseHandler) {
        match msg.text() {
            Some(KEY_SANTA_CHAT_CLOSE) => {
                let keyboard = KeyboardMarkup::new([
                    [KeyboardButton::new(KEY_CHILD_CHAT)],
                    [KeyboardButton::new(KEY_SANTA_CHAT)],
                ])
                .resize_keyboard(true);

                bot.send_message(msg.chat.id, "Можешь написать подопечнму или Санте:")
                    .reply_markup(keyboard)
                    .await
                    .unwrap();
                user.state = Option::from(State::Distributed);
                db.save_user(user).await;
            }
            Some(message_text) => {
                let santa_id = user.santa.unwrap();
                let message = types::Message {
                    child_id: user.id,
                    santa_id,
                    message: format!("<b>$child: </b>\n{}", message_text),
                    create_date: DateTimeWithTimeZone::from(Utc::now()),
                };
                db.save_message(message).await;

                bot.send_message(
                    ChatId(santa_id),
                    format!("У вас новое сообщение от подопечного:\n{}", message_text),
                )
                .await
                .unwrap();
            }
            None => {}
        }
    }
}

#[async_trait]
impl StateStrategy for ReceiveWishStrategy {
    async fn handle(&self, mut user: User, msg: Message, bot: SantaBot, db: DatabaseHandler) {
        match msg.text() {
            Some(wish) => {
                let inline_keyboard = InlineKeyboardMarkup::new([
                    [InlineKeyboardButton::callback(
                        MOSCOW_CITY,
                        CITY_CALLBACK_MSK,
                    )],
                    [InlineKeyboardButton::callback(
                        IZHEVSK_CITY,
                        CITY_CALLBACK_IZH,
                    )],
                ]);

                bot.send_message(
                    msg.chat.id,
                    include_str!("templates/stat_3_select_city_0.txt"),
                )
                .disable_web_page_preview(true)
                .reply_markup(inline_keyboard)
                .await
                .unwrap();

                user.wish_text = wish.parse().unwrap();
                user.state = Option::from(State::ReceiveCity);
                db.save_user(user).await;
            }
            None => {
                bot.send_message(msg.chat.id, "Отправьте мне обычный текст.")
                    .await
                    .unwrap();
            }
        }
    }
}

#[async_trait]
impl StateStrategy for ReceiveNameStrategy {
    async fn handle(&self, mut user: User, msg: Message, bot: SantaBot, db: DatabaseHandler) {
        match msg.text() {
            Some(username) => {
                user.username = username.parse().unwrap();
                user.state = Option::from(State::ReceiveWish);

                bot.send_message(
                    msg.chat.id,
                    include_str!("templates/state_2_write_name_0.txt"),
                )
                .await
                .unwrap();

                let url_state_2 =
                    "https://www.sunhome.ru/i/cards/198/elka-animacionnaya-otkritka.orig.gif";
                bot.send_animation(
                    msg.chat.id,
                    InputFile::url(Url::parse(url_state_2).unwrap()),
                )
                .disable_notification(true)
                .await
                .unwrap();

                bot.send_message(
                    msg.chat.id,
                    include_str!("templates/state_2_write_name_1.txt"),
                )
                .await
                .unwrap();
                db.find_user(2).await;
                db.save_user(user).await;
            }
            None => {
                bot.send_message(msg.chat.id, "Отправьте мне обычный текст.")
                    .await
                    .unwrap();
            }
        }
    }
}
