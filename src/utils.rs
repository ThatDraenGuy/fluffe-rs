use teloxide::types::Message;

pub const DEFAULT_MENTION: &str = "@ThatDraenGuy";
pub const DEFAULT_LOCALE: &str = "ru";

pub fn is_mention(arg: &str) -> bool {
    arg.starts_with('@') && !arg.contains(char::is_whitespace)
}

pub fn get_language_code(msg: &Message) -> &str {
    msg.from()
        .and_then(|user| user.language_code.as_ref())
        .map_or(DEFAULT_LOCALE, |code| code.as_str())
}
