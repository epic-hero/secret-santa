use fmt::Debug;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;

use reqwest::Url;
use strum_macros::EnumString;
use teloxide::prelude::*;
use teloxide::types::{InputFile, KeyboardButton, KeyboardMarkup};
use teloxide::utils::command::BotCommands;

use crate::types::User;
use crate::{db, SantaBot};

pub const IZHEVSK_CITY: &str = "Ижевск";
pub const MOSCOW_CITY: &str = "Москва";
pub const CITY_CALLBACK_IZH: &str = "__izh_callback";
pub const CITY_CALLBACK_MSK: &str = "__msk_callback";
pub const CHANGE_WISH_CALLBACK: &str = "__change_wish_callback";

pub const SANTA_PATTERN: &'static str = "$santa";
pub const CHILD_PATTERN: &'static str = "$child";
pub const CHANGE_WISH_LIST: &str = "🎁 Обновить список желаний";
pub const KEY_CHILD_CHAT: &str = "🏠 Перейти к беседе с подопечным";
pub const KEY_SANTA_CHAT: &str = "🎅 Перейти к беседе с Сантой";
pub const KEY_CHILD_CHAT_CLOSE: &str = "Закрыть чат с подопечным";
pub const KEY_SANTA_CHAT_CLOSE: &str = "Закрыть чат с Сантой";
pub const ADMIN_ID: i64 = 628456869;

#[derive(BotCommands, Clone, Default, Debug, EnumString, PartialEq)]
pub enum State {
    #[default]
    Start,
    ReceiveName,
    ReceiveWish,
    ReceiveCity,
    ChildChat,
    SantaChat,
    ChangeWishList,
    Finish,
    Distributed,
}

impl Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(BotCommands, Clone, Debug)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command(description = "Запуск бота")]
    Start,
    #[command(description = "help")]
    Help,
    #[command(description = "Получить список авторизованных пользователей")]
    List,
    #[command(description = "Распредилить подопечных")]
    Distribute,
    #[command(description = "Уведомить пользователей о санте")]
    Notify,
}

#[derive(Debug)]
pub struct MyBot {}

impl MyBot {
    pub async fn new() -> Self {
        MyBot {}
    }

    pub async fn send_start(&self, bot: SantaBot, msg: Message) -> ResponseResult<()> {
        bot.send_message(
            msg.chat.id,
            "А теперь, внучок, представься и назови мне свое имя",
        )
        .await?;
        Ok(())
    }

    pub async fn send_list_users(
        &self,
        bot: &SantaBot,
        msg: &Message,
        db: &db::DatabaseHandler,
    ) -> ResponseResult<()> {
        let users = db.get_all_users().await;
        bot.send_message(msg.chat.id, format!("Users: {:?}", users))
            .await?;
        Ok(())
    }

    pub async fn notify(&self, bot: &SantaBot, db: &db::DatabaseHandler) -> ResponseResult<()> {
        let users = db
            .get_all_users()
            .await
            .iter()
            .map(|user| (user.chat_id, user.clone()))
            .collect::<HashMap<i64, User>>();

        for (_, mut user) in users.clone().into_iter() {
            match user.child {
                Some(child) => {
                    let child = users.get(&child).unwrap();
                    let response_msg =
                        format!(include_str!("templates/state_5_notify.txt"), child.username);
                    bot.send_message(ChatId(user.chat_id), response_msg).await?;
                    let url_state_1 = "https://media1.giphy.com/media/v1.Y2lkPTc5MGI3NjExMzZ4cTlpMm1nMXd6NWIzZTlnZW45YXM4dTByeWc1OWQzbXhtNXI3NCZlcD12MV9pbnRlcm5hbF9naWZfYnlfaWQmY3Q9cw/63Iznk0GDRB4U8f07H/giphy.gif";
                    bot.send_animation(
                        ChatId(user.chat_id),
                        InputFile::url(Url::parse(url_state_1).unwrap()),
                    )
                    .disable_notification(true)
                    .await?;
                    let response_msg = format!(
                        include_str!("templates/state_5_notify_1.txt"),
                        child.wish_text
                    );
                    bot.send_message(ChatId(user.chat_id), response_msg).await?;

                    send_keyboard(&bot, ChatId(user.chat_id)).await?;
                    user.state = Option::from(State::Distributed);
                    db.save_user(user).await;
                }
                None => {}
            }
        }
        Ok(())
    }

    pub async fn distribute(
        &self,
        bot: &SantaBot,
        msg: &Message,
        db: &db::DatabaseHandler,
    ) -> ResponseResult<()> {
        let users: Vec<User> = db.get_all_users().await;
        let mut izhevsk: Vec<User> = vec![];
        let mut moscow: Vec<User> = vec![];

        users.into_iter().for_each(|user| match user.city.as_str() {
            IZHEVSK_CITY => izhevsk.push(user),
            MOSCOW_CITY => moscow.push(user),
            _ => {}
        });

        if izhevsk.len() != moscow.len() {
            let response_msg = format!("Кол-во людей в команде должно быть одинаковым");
            bot.send_message(msg.chat.id, response_msg).await?;
            return Ok(());
        }
        Self::_distribute(&mut izhevsk, &mut moscow);
        moscow.reverse();
        Self::_distribute(&mut moscow, &mut izhevsk);
        let ready_users = [izhevsk, moscow].concat();
        let response_msg = format!("Распредилил роли, кол-во человек: {}", ready_users.len());
        db.save_users(ready_users).await;

        bot.send_message(msg.chat.id, response_msg).await?;

        let users = db
            .get_all_users()
            .await
            .iter()
            .map(|user| (user.chat_id, user.clone()))
            .collect::<HashMap<i64, User>>();

        for (_, user) in users.iter() {
            bot.send_message(ChatId(user.chat_id), "Хо-хо-хо! Уже сегодня мы узнаем, какому счастливчику ты сделаешь самый лучший новогодний подарок!").await?;
        }
        Ok(())
    }

    fn _distribute(first_group: &mut Vec<User>, second_group: &mut Vec<User>) {
        for user_first in first_group.iter_mut() {
            for user_second in second_group.iter_mut() {
                if user_first.santa.is_none() && user_second.child.is_none() {
                    user_first.set_santa(user_second.id);
                    user_second.set_child(user_first.id);
                    user_second.state = Option::from(State::Distributed);
                }
            }
        }
    }

    pub async fn send_help(&self, bot: SantaBot, msg: Message) -> ResponseResult<()> {
        bot.send_message(
            msg.chat.id,
            "Не разобрался и не разобрался, с кем не бывает!",
        )
        .await?;
        Ok(())
    }
}

async fn send_keyboard(bot: &SantaBot, chat_id: ChatId) -> ResponseResult<()> {
    let keyboard = KeyboardMarkup::new([
        [KeyboardButton::new(KEY_CHILD_CHAT)],
        [KeyboardButton::new(KEY_SANTA_CHAT)],
    ])
    .resize_keyboard(true);

    bot.send_message(chat_id, "Можешь перейти к беседе с подопечным или Сантой:")
        .reply_markup(keyboard)
        .await?;

    Ok(())
}
