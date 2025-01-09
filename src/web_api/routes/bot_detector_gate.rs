use std::future::{ready, Ready};

use actix_web::{dev::Payload, error::Error as ActixWebError, http, FromRequest, HttpRequest};

pub struct BotDetector {
    pub is_bot: bool,
}

impl FromRequest for BotDetector {
    type Error = ActixWebError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let user_agent_header_opt = req.headers().get(http::header::USER_AGENT);
        let user_agent_value =
            user_agent_header_opt.map(|user_agent| user_agent.to_str().unwrap_or_default());
        let is_bot = user_agent_value
            .map(|value| {
                value.to_lowercase().contains("bot")
                    || value.to_lowercase().contains("crawler")
                    || value.to_lowercase().contains("spider")
                    || value.to_lowercase().contains("crawling")
            })
            .unwrap_or_default();
        ready(Ok(BotDetector { is_bot }))
    }
}
