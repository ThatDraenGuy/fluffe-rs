#[macro_use]
extern crate rust_i18n;

i18n!("locales", fallback = "ru");

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    dotenvy::dotenv().ok();
    rust_i18n::set_locale("ru");
}
