use crate::db::DatabaseHandler;
use crate::states::state_factory;
use crate::types::User;
use crate::SantaBot;
use teloxide::prelude::*;

pub async fn handle_message(
    db: DatabaseHandler,
    bot: SantaBot,
    msg: Message,
) -> ResponseResult<()> {
    let user = get_user(&db, &msg).await;
    let state = state_factory(&user.state);
    state.as_ref().handle(user, msg, bot, db).await;
    Ok(())
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
