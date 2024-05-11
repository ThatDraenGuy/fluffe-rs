use teloxide::{
    payloads::SendMessageSetters,
    requests::{Requester, ResponseResult},
    types::{ChatId, Message},
};

use crate::{
    consts::{DEFAULT_LOCALE, DEFAULT_MENTION},
    AppError, AppResult, ClientError, FluffersBot,
};

pub fn is_mention(arg: &str) -> bool {
    arg.starts_with('@') && !arg.contains(char::is_whitespace)
}

pub fn resolve_mention<'a>(arg: &'a str, cmd: &'static str) -> AppResult<&'a str> {
    if is_mention(arg) {
        Ok(arg.split_at(1).1)
    } else {
        Err(ClientError::NoMention(cmd).into())
    }
}

pub fn get_language_code(msg: &Message) -> &str {
    msg.from()
        .and_then(|user| user.language_code.as_ref())
        .map_or(DEFAULT_LOCALE, |code| code.as_str())
}

pub async fn send_error_msg(
    bot: &FluffersBot,
    chat_id: ChatId,
    locale: &str,
    src: Option<&Message>,
    e: &AppError,
) -> ResponseResult<()> {
    let mut send = bot.send_message(
        chat_id,
        match e {
            AppError::ClientError(cli_err) => match cli_err {
                ClientError::NoMention(cmd) => t!(
                    "msg.common.error.client.mention_argument",
                    command = cmd,
                    locale = locale,
                    mention = DEFAULT_MENTION
                ),
                ClientError::NoUser(username) => t!(
                    "msg.common.error.client.unknown_username",
                    locale = locale,
                    mention = username,
                ),
            },
            AppError::UnknownPlayer => {
                t!("msg.common.error.server.unknown_player", locale = locale,)
            }
            AppError::Database(db_err) => t!(
                "msg.common.error.server.db_err",
                locale = locale,
                msg = db_err
            ),
            e => t!(
                "msg.common.error.server.unknown_err",
                locale = locale,
                msg = e
            ),
        },
    );

    if let Some(msg) = src {
        send = send.reply_to_message_id(msg.id);
    }

    send.await?;
    Ok(())
}
