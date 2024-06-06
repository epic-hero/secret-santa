use std::convert::Infallible;
use std::env;
use std::net::SocketAddr;

use crate::bot::Command;
use teloxide::adaptors::trace::Settings;
use teloxide::adaptors::{DefaultParseMode, Trace};
use teloxide::update_listeners::UpdateListener;
use teloxide::{prelude::*, update_listeners::webhooks};

use crate::db::DatabaseHandler;
use crate::hendlers::handle_callback_query::handle_callback_query;
use crate::hendlers::handle_command::handle_command;
use crate::hendlers::handle_message::handle_message;

mod bot;
mod db;
mod hendlers;
mod states;
mod types;

pub type SantaBot = DefaultParseMode<Trace<Bot>>;

#[tokio::main]
async fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    log::info!("Starting bot...");
    pretty_env_logger::init();

    build_dispatcher().await;
}

#[cfg(debug_assertions)]
async fn build_dispatcher() {
    // Для локального запуска (long polling)
    let db = DatabaseHandler::from_env().await;
    let bot = Bot::from_env()
        .trace(Settings::TRACE_EVERYTHING_VERBOSE)
        .parse_mode(teloxide::types::ParseMode::Html);

    return Dispatcher::builder(
        bot,
        dptree::entry()
            .branch(
                Update::filter_message()
                    .filter_command::<Command>()
                    .endpoint(handle_command),
            )
            .branch(Update::filter_message().endpoint(handle_message))
            .branch(Update::filter_callback_query().endpoint(handle_callback_query)),
    )
        .dependencies(dptree::deps![db])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

#[cfg(not(debug_assertions))]
async fn build_dispatcher() {
    // Для релизного запуска (webhooks)
    let db = DatabaseHandler::from_env().await;
    let bot = Bot::from_env()
        .trace(Settings::TRACE_EVERYTHING_VERBOSE)
        .parse_mode(teloxide::types::ParseMode::Html);

    let listener = get_listener(bot.clone()).await;

    Dispatcher::builder(
        bot,
        dptree::entry()
            .branch(
                Update::filter_message()
                    .filter_command::<Command>()
                    .endpoint(handle_command),
            )
            .branch(Update::filter_message().endpoint(handle_message))
            .branch(Update::filter_callback_query().endpoint(handle_callback_query)),
    )
        .dependencies(dptree::deps![db])
        .enable_ctrlc_handler()
        .build()
        .dispatch_with_listener(
            listener,
            LoggingErrorHandler::with_custom_text("An error from the update listener"),
        )
        .await;
}

#[cfg(not(debug_assertions))]
async fn get_listener(bot: SantaBot) -> impl UpdateListener<Err=Infallible> + Sized {
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
