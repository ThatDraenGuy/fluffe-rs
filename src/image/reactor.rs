use html5tokenizer::{NaiveParser, Token};
use reqwest::Url;
use teloxide::types::InputFile;

use crate::{AppError, AppResult};

use super::ImageRepositoryTrait;

const BASE_URL: &str = "https://furry.reactor.cc/";

pub struct ReactorRepository {
    base_url: Url,
}

impl ReactorRepository {
    fn scrape_for_link(&self, html: &str) -> AppResult<Url> {
        for token in NaiveParser::new(html).flatten() {
            let Token::StartTag(tag) = token else {
                continue;
            };

            let attrs = tag.attributes;

            let Some(value) = attrs.get("href").or(attrs.get("src")) else {
                continue;
            };
            if value.contains("pics/post") && !value.contains("/full/") {
                let link = if value.starts_with("https:") {
                    value.to_owned()
                } else {
                    format!("https://{}", value)
                };
                log::info!("Found reactor image url: {}", link);
                return Ok(Url::parse(link.as_str())?);
            }
        }

        Err(AppError::NoImageFound)
    }

    async fn get_image_from_post(&self, post_path: &str) -> AppResult<Url> {
        let response = reqwest::get(self.base_url.join(post_path)?)
            .await?
            .error_for_status()?;

        let body = response.text().await?;
        self.scrape_for_link(&body)
    }
}

impl Default for ReactorRepository {
    fn default() -> Self {
        Self {
            base_url: Url::parse(BASE_URL).expect("Invalid reactor url set!"),
        }
    }
}

impl ImageRepositoryTrait for ReactorRepository {
    async fn get_random_image(&self) -> AppResult<InputFile> {
        Ok(InputFile::url(self.get_image_from_post("/random").await?))
    }
}
