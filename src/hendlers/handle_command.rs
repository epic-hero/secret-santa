use reqwest::Url;
use teloxide::payloads::SendAnimationSetters;
use teloxide::prelude::{Message, Requester, ResponseResult};
use teloxide::types::InputFile;
use teloxide::utils::command::BotCommands;

use crate::bot::{Command, MyBot, State, ADMIN_ID};
use crate::db::DatabaseHandler;
use crate::types::User;
use crate::SantaBot;

pub async fn handle_command(
    db: DatabaseHandler,
    bot: SantaBot,
    msg: Message,
) -> ResponseResult<()> {
    let my_bot = MyBot::new().await;
    let mut user = get_user(&db, &msg).await;

    match BotCommands::parse(msg.text().unwrap(), "") {
        Ok(Command::Start) => {
            if user.wish_text != "" {
                bot.send_message(msg.chat.id, "Хитрец! Больше одного подарка не положено")
                    .await?;
                return Ok(());
            }
            user.state = Option::from(State::ReceiveName);
            db.save_user(user).await;

            bot.send_message(
                msg.chat.id,
                "Хо-хо-хо! Приветствую тебя в тайном чате Бай Семьи!",
            )
            .await?;
            let url_state_1 =
                "https://media.baamboozle.com/uploads/images/153804/1608041764_533336";
            bot.send_animation(
                msg.chat.id,
                InputFile::url(Url::parse(url_state_1).unwrap()),
            )
            .disable_notification(true)
            .await?;
            bot.send_message(msg.chat.id, "А почему он тайный, спросишь ты? Потому что здесь мы распределяем Тайных Сант на самый волшебный праздник – Новый Год 🎅🎄 ").await?;

            my_bot.send_start(bot, msg).await?;
        }
        Ok(Command::List) => {
            if user.chat_id == ADMIN_ID {
                my_bot.send_list_users(&bot, &msg, &db).await?;
            }
        }
        Ok(Command::Notify) => {
            if user.chat_id == ADMIN_ID {
                my_bot.notify(&bot, &db).await?;
            }
        }
        Ok(Command::Distribute) => {
            if user.chat_id == ADMIN_ID {
                my_bot.distribute(&bot, &msg, &db).await?;
            }
        }
        Ok(Command::Help) => {
            my_bot.send_help(bot, msg).await?;
        }
        _ => {}
    }
    return Ok(());
}

async fn get_user(db: &DatabaseHandler, msg: &Message) -> User {
    let user_id = msg.chat.id.0;
    let name = msg
        .chat
        .username()
        .or(msg.chat.first_name())
        .unwrap_or(user_id.to_string().as_str())
        .to_string();
    db.get_user(user_id)
        .await
        .or(User::default_user(user_id, name))
        .unwrap()
}
