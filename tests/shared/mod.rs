use grammers_client::{Client, Config};
use grammers_session::{PackedChat, PackedType, Session};
use log::info;
use simple_logger::SimpleLogger;
use crate::shared::utils::auth;

pub mod messages;
pub mod utils;

static TG_API_ID: &str = env!("TG_API_ID");
static TG_API_HASH: &str = env!("TG_API_HASH");
static BOT_ACCESS_HASH: &str = env!("BOT_ACCESS_HASH");
static BOT_ID: &str = env!("BOT_ID");
static SESSION_FILE: &str = env!("SESSION_FILE");

pub async fn get_client() -> Client {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .unwrap();

    let api_id = TG_API_ID.parse().expect("TG_ID invalid");
    let api_hash = TG_API_HASH.to_string();

    info!("Connecting to Telegram...");
    let config = Config {
        session: Session::load_file_or_create(SESSION_FILE).unwrap(),
        api_id,
        api_hash: api_hash.clone(),
        params: Default::default(),
    };
    let client = Client::connect(config).await.unwrap();
    if !client.is_authorized().await.unwrap() && !auth(&client).await {
        drop(client.sign_out_disconnect().await);
    }
    client
}

pub fn get_bot_chat() -> PackedChat {
    let access_hash = BOT_ACCESS_HASH
        .parse()
        .expect("env BOT_ACCESS_HASH not found");
    let id = BOT_ID.parse().expect("env BOT_ID not found");
    PackedChat {
        ty: PackedType::Bot,
        id,
        access_hash: Some(access_hash),
    }
}
