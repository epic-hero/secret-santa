mod shared;

use crate::shared::messages::{
    BOT_START_COMMAND, BOT_USERNAME_MESSAGE, BOT_WISH_MESSAGE, CITY_CALLBACK_MSK,
    RECEIVE_CITY_RESPONSE, RECEIVE_NAME_RESPONSE, RECEIVE_WISH_RESPONSE, REPEAT_START_MESSAGE,
    START_MESSAGES_RESPONSE,
};
use crate::shared::utils::{check_messages, get_last_message};
use crate::shared::{get_bot_chat, get_client};
use grammers_client::types::Message;
use grammers_client::Client;
use grammers_session::PackedChat;
use grammers_tl_types::enums::InputPeer;
use grammers_tl_types::functions::messages::GetBotCallbackAnswer;
use grammers_tl_types::types::InputPeerUser;

#[tokio::test]
async fn integration_test() {
    let client = get_client().await;
    let bot_chat = get_bot_chat();
    BotSuccessTest::new(client, bot_chat).run().await;
}

pub struct BotSuccessTest {
    client: Client,
    chat: PackedChat,
}

impl BotSuccessTest {
    pub fn new(client: Client, chat: PackedChat) -> Self {
        Self { client, chat }
    }

    pub async fn run(&self) {
        self.client.delete_dialog(self.chat).await.unwrap();
        self.start_command_test().await;
        self.receive_name_test().await;
        self.receive_wish_test().await;
        self.repeat_start_command_test().await;
    }

    /// Первое знакомство с ботом
    pub async fn start_command_test(&self) {
        self.send_message(BOT_START_COMMAND).await;
        check_messages(START_MESSAGES_RESPONSE.to_vec(), &self.client).await;
    }

    /// Повторный запрос на /start
    pub async fn repeat_start_command_test(&self) {
        self.send_message(BOT_START_COMMAND).await;
        check_messages(vec![REPEAT_START_MESSAGE], &self.client).await;
    }

    /// Отправляем имя участника
    pub async fn receive_name_test(&self) {
        self.send_message(BOT_USERNAME_MESSAGE).await;
        check_messages(RECEIVE_NAME_RESPONSE.to_vec(), &self.client).await;
    }

    /// Отправляем пожелания
    async fn receive_wish_test(&self) {
        self.send_message(BOT_WISH_MESSAGE).await;
        let response = get_last_message(&self.client).await.unwrap();
        assert_eq!(RECEIVE_WISH_RESPONSE, response.text());
        self.receive_city_test(response).await;
    }

    /// Выбор города
    async fn receive_city_test(&self, message: Message) {
        let answer = GetBotCallbackAnswer {
            game: false,
            peer: InputPeer::User(InputPeerUser {
                user_id: self.chat.id,
                access_hash: self.chat.access_hash.unwrap(),
            }),
            msg_id: message.id(),
            data: Some(Vec::from(CITY_CALLBACK_MSK)),
            password: None,
        };

        let _ = &self
            .client
            .invoke(&answer)
            .await
            .expect("Нет ответа на callback");

        check_messages(RECEIVE_CITY_RESPONSE.to_vec(), &self.client).await;
    }

    async fn send_message(&self, message: &str) {
        let _ = &self.client.send_message(self.chat, message).await.unwrap();
    }
}
