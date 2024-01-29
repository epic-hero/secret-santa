use std::convert::Infallible;
use std::env;
use std::net::SocketAddr;

use teloxide::{prelude::*, update_listeners::webhooks};
use teloxide::adaptors::DefaultParseMode;
use teloxide::update_listeners::UpdateListener;

use crate::db::DatabaseHandler;
use crate::handle_callback_query::handle_callback_query;
use crate::handle_message::handle_message;

mod handle_message;
mod handle_callback_query;

mod types;
mod bot;
mod db;

pub type DLEBot = DefaultParseMode<Bot>;

#[tokio::main]
async fn main() {
    log::info!("Starting bot...");
    pretty_env_logger::init();

    let db = DatabaseHandler::from_env().await;
    let bot = Bot::from_env()
        .parse_mode(teloxide::types::ParseMode::Html);

    let listener = get_listener(bot.clone()).await;

    Dispatcher::builder(bot, dptree::entry()
        .branch(Update::filter_message().endpoint(handle_message))
        .branch(Update::filter_callback_query().endpoint(handle_callback_query)))
        .dependencies(dptree::deps![db])
        .enable_ctrlc_handler()
        .build()
        .dispatch_with_listener(
            listener,
            LoggingErrorHandler::with_custom_text("An error from the update listener"),
        )
        .await;
}

async fn get_listener(bot: DefaultParseMode<Bot>) -> impl UpdateListener<Err=Infallible> + Sized {
    let addr: SocketAddr = env::var("APP_HOST")
        .expect("APP_HOST not found")
        .parse()
        .expect("Unable to parse APP_HOST to SocketAddr");

    let listen_url = env::var("LISTEN_URL")
        .expect("LISTEN_URL not found")
        .parse()
        .expect("Unable to parse LISTEN_URL");
    let listener = webhooks::axum(bot, webhooks::Options::new(addr, listen_url))
        .await
        .expect("Couldn't setup webhook");
    listener
}
