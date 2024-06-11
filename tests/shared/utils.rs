use std::io;
use std::io::{BufRead, Write};
use std::time::Duration;

use crate::shared::SESSION_FILE;
use grammers_client::types::Message;
use grammers_client::{Client, SignInError, Update};
use tokio::time::timeout;

const TIMEOUT_SECS_TG_REQUEST: u64 = 1;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub(crate) async fn check_messages(expected_messages: Vec<&str>, client: &Client) {
    for expected_message in expected_messages {
        let response = get_last_message(&client).await.unwrap();
        assert_eq!(expected_message, response.text());
    }
}

pub(crate) async fn get_last_message(client: &Client) -> Option<Message> {
    while let Some(update) = next_update(&client).await {
        match update {
            Update::NewMessage(message) if !message.outgoing() => {
                return Some(message);
            }
            Update::MessageEdited(message) => {
                return Some(message);
            }
            _ => {}
        }
    }
    return None;
}

async fn next_update(client: &Client) -> Option<Update> {
    timeout(
        Duration::from_secs(TIMEOUT_SECS_TG_REQUEST),
        client.next_update(),
    )
    .await
    .expect("Таймаут!! Не удалось получить сообщение из телеграм.")
    .unwrap()
}

pub(crate) async fn auth(client: &Client) -> bool {
    println!("Signing in...");
    let phone = prompt("Enter your phone number (international format): ").unwrap();
    let token = client.request_login_code(&phone).await.unwrap();
    let code = prompt("Enter the code you received: ").unwrap();
    let signed_in = client.sign_in(&token, &code).await;
    match signed_in {
        Err(SignInError::PasswordRequired(password_token)) => {
            let hint = password_token.hint().unwrap_or("None");
            let prompt_message = format!("Enter the password (hint {}): ", &hint);
            let password = prompt(prompt_message.as_str()).unwrap();

            client
                .check_password(password_token, password.trim())
                .await
                .unwrap();
        }
        Ok(_) => (),
        Err(e) => panic!("{}", e),
    };
    println!("Signed in!");
    match client.session().save_to_file(SESSION_FILE) {
        Ok(_) => {}
        Err(e) => {
            println!(
                "NOTE: failed to save the session, will sign out when done: {}",
                e
            );
            return false;
        }
    }
    return true;
}

fn prompt(message: &str) -> Result<String> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    stdout.write_all(message.as_bytes()).unwrap();
    stdout.flush().unwrap();

    let stdin = io::stdin();
    let mut stdin = stdin.lock();

    let mut line = String::new();
    stdin.read_line(&mut line).unwrap();
    Ok(line)
}
