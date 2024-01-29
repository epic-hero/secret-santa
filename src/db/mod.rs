use std::env;

use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectOptions, Database, DatabaseConnection, DbBackend, EntityTrait, QueryFilter, Statement};
use sea_orm::ActiveValue::Set;

use crate::bot::State;
use crate::db::schema::{message, user};
use crate::db::schema::user::Model;
use crate::types::{Message, User};

pub(crate) mod schema;

#[derive(Clone)]
pub struct DatabaseHandler {
    db: DatabaseConnection,
}

impl DatabaseHandler {
    pub async fn new(uri: String) -> Self {
        let mut opt = ConnectOptions::new(uri);
        opt.sqlx_logging(false);

        let db = Database::connect(opt).await.unwrap();

        DatabaseHandler { db }
    }

    pub async fn from_env() -> Self {
        Self::new(env!("DATABASE_URL").to_string()).await
    }

    pub async fn save_users(&self, user_dtos: Vec<User>) {
        for user in user_dtos.into_iter() {
            self.save_user(user).await;
        }
    }

    pub async fn save_message(&self, message_dto: Message) {
        if let Some(message_db) = self.find_message(message_dto.santa_id, message_dto.child_id).await {
            let mut message: message::ActiveModel = message_db.into();
            message.message = Set(format!("{}\n{}", message.message.unwrap(), message_dto.message));
            if let Err(x) = message.update(&self.db).await {
                log::error!("Error accessing the database: {:?}", x);
            }
        } else {
            let new_message = message::ActiveModel {
                id: Default::default(),
                santa_id: Set(message_dto.santa_id),
                child_id: Set(message_dto.child_id),
                message: Set(message_dto.message),
                create_date: Set(message_dto.create_date),
            };
            if let Err(x) = new_message.insert(&self.db).await {
                log::error!("Error accessing the database: {:?}", x);
            };
        }
    }

    pub async fn save_user(&self, user_dto: User) {
        if let Some(user) = self.find_user(user_dto.id).await {
            let mut user: user::ActiveModel = user.into();
            user.username = Set(user_dto.username);
            user.wish_text = Set(user_dto.wish_text);
            user.city = Set(user_dto.city);
            user.child = Set(user_dto.child);
            user.santa = Set(user_dto.santa);
            user.state = Set(match user_dto.state {
                Some(state) => { state.to_string() }
                _ => { "".to_string() }
            });
            if let Err(x) = user.update(&self.db).await {
                log::error!("Error accessing the database: {:?}", x);
            }
        } else {
            let new_user = user::ActiveModel {
                id: Set(user_dto.id),
                chat_id: Set(user_dto.chat_id),
                child: Default::default(),
                santa: Default::default(),
                nickname: Set(user_dto.nickname),
                username: Set(user_dto.username),
                city: Set(user_dto.city),
                wish_text: Set(user_dto.wish_text),
                state: Set(user_dto.state.or(Option::from(State::default())).unwrap().to_string()),
                create_date: Set(user_dto.create_date),
            };
            if let Err(x) = new_user.insert(&self.db).await {
                log::error!("Error accessing the database: {:?}", x);
            };
        }
    }

    pub async fn find_message(&self, santa_id: i64, child_id: i64) -> Option<message::Model> {
        message::Entity::find()
            .filter(message::Column::SantaId.eq(santa_id))
            .filter(message::Column::ChildId.eq(child_id))
            .one(&self.db)
            .await
            .unwrap_or_else(|x| {
                log::error!("Error accessing the database: {:?}", x);
                None
            })
    }
    pub async fn find_user(&self, user_id: i64) -> Option<Model> {
        user::Entity::find()
            .filter(user::Column::Id.eq(user_id))
            .one(&self.db)
            .await
            .unwrap_or_else(|x| {
                log::error!("Error accessing the database: {:?}", x);
                None
            })
    }
    pub async fn get_user(&self, user_id: i64) -> Option<User> {
        self.find_user(user_id).await
            .map(|user| User::to_user(user))
    }

    pub async fn get_all_users(&self) -> Vec<User> {
        user::Entity::find()
            .from_raw_sql(Statement::from_string(
                DbBackend::Postgres,
                r#"select id, chat_id, nickname, username, wish_text, state, city, child, santa, create_date from public.user"#.to_string()))
            .all(&self.db)
            .await
            .map(|user| User::to_users(user))
            .unwrap_or_else(|x| {
                log::error!("Error accessing the database: {:?}", x);
                vec![]
            })
    }
}
