use std::str::FromStr;

use chrono::Utc;
use sea_orm::prelude::DateTimeWithTimeZone;

use crate::bot::State;
use crate::db::schema::user::Model;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct User {
    pub id: i64,
    pub chat_id: i64,
    pub child: Option<i64>,
    pub santa: Option<i64>,
    pub nickname: String,
    pub username: String,
    pub wish_text: String,
    pub city: String,
    pub state: Option<State>,
    pub create_date: DateTimeWithTimeZone,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Message {
    pub santa_id: i64,
    pub child_id: i64,
    pub message: String,
    pub create_date: DateTimeWithTimeZone,
}

impl User {
    pub fn to_users(users: Vec<Model>) -> Vec<User> {
        users.into_iter()
            .map(|user| User::to_user(user))
            .collect()
    }
    pub fn to_user(user: Model) -> User {
        User {
            id: user.id,
            chat_id: user.chat_id,
            child: user.child,
            santa: user.santa,
            nickname: user.nickname,
            username: user.username,
            wish_text: user.wish_text,
            city: user.city,
            state: State::from_str(user.state.as_str()).ok(),
            create_date: user.create_date,
        }
    }
    pub fn default_user(chat_id: i64, nickname: String) -> Option<User> {
        let create_date = DateTimeWithTimeZone::from(Utc::now());
        Option::from(User {
            id: chat_id,
            nickname,
            chat_id,
            create_date,
            ..Default::default()
        })
    }
    pub fn set_santa(&mut self, santa_id: i64) {
        self.santa = Option::from(santa_id);
    }

    pub fn set_child(&mut self, child_id: i64) {
        self.child = Option::from(child_id);
    }
}
