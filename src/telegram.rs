use anyhow::{Context, Result, bail};
use serde::Deserialize;
use serde::de::DeserializeOwned;

const DEFAULT_BASE_URL: &str = "https://api.telegram.org";

fn redact_token(text: &str, token: &str) -> String {
    // If token is empty, `replace` would insert <redacted> between every character.
    if token.is_empty() {
        return text.to_string();
    }
    text.replace(token, "<redacted>")
}

pub struct TelegramClient {
    http: reqwest::Client,
    base_url: String,
    token: String,
}

impl TelegramClient {
    pub fn new(token: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: DEFAULT_BASE_URL.to_string(),
            token,
        }
    }

    fn method_url(&self, method: &str) -> String {
        format!("{}/bot{}/{}", self.base_url, self.token, method)
    }

    fn reqwest_error(&self, method: &str, e: reqwest::Error) -> anyhow::Error {
        // reqwest::Error Display often includes the full request URL; for Telegram this
        // contains the bot token, so we must redact it.
        let msg = redact_token(&e.to_string(), &self.token);
        anyhow::anyhow!("telegram request failed: method={method}: {msg}")
    }

    async fn post_json<T: DeserializeOwned>(
        &self,
        method: &str,
        body: serde_json::Value,
    ) -> Result<T> {
        let url = self.method_url(method);

        let res = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| self.reqwest_error(method, e))?;

        let status = res.status();
        let text = res
            .text()
            .await
            .map_err(|e| self.reqwest_error(method, e))?;

        if !status.is_success() {
            bail!("telegram http error: method={method} status={status} body={text}");
        }

        let parsed: ApiResponse<T> = serde_json::from_str(&text)
            .with_context(|| format!("parse telegram response json: {method}"))?;

        parsed
            .into_result()
            .with_context(|| format!("telegram method failed: {method}"))
    }

    pub async fn send_message(&self, user_id: i64, text: &str) -> Result<i64> {
        #[derive(Deserialize)]
        struct SendMessageResult {
            message_id: i64,
        }

        let result: SendMessageResult = self
            .post_json(
                "sendMessage",
                serde_json::json!({
                    "chat_id": user_id,
                    "text": text,
                }),
            )
            .await?;

        Ok(result.message_id)
    }

    pub async fn get_updates(&self, offset: i64, timeout_s: u64) -> Result<Vec<Update>> {
        let mut body = serde_json::Map::new();
        body.insert("offset".to_string(), serde_json::json!(offset));
        body.insert("timeout".to_string(), serde_json::json!(timeout_s));
        body.insert(
            "allowed_updates".to_string(),
            serde_json::json!(["message"]),
        );

        self.post_json("getUpdates", serde_json::Value::Object(body))
            .await
    }

    pub async fn drain_updates(&self) -> Result<i64> {
        let mut offset: i64 = 0;

        loop {
            let updates = self.get_updates(offset, 0).await?;
            if updates.is_empty() {
                return Ok(offset);
            }

            let last = updates.last().expect("non-empty updates").update_id;
            offset = last + 1;
        }
    }
}

#[derive(Debug, Deserialize)]
struct ApiResponse<T> {
    ok: bool,
    result: Option<T>,
    description: Option<String>,
    error_code: Option<i64>,
}

impl<T> ApiResponse<T> {
    fn into_result(self) -> Result<T> {
        if self.ok {
            return self
                .result
                .context("telegram response missing result despite ok=true");
        }

        let code = self.error_code.unwrap_or(0);
        let desc = self
            .description
            .unwrap_or_else(|| "unknown telegram error".to_string());
        bail!("telegram api error {code}: {desc}")
    }
}

#[derive(Debug, Deserialize)]
pub struct Update {
    pub update_id: i64,
    pub message: Option<Message>,
}

#[derive(Debug, Deserialize)]
pub struct Message {
    pub from: Option<User>,
    pub chat: Chat,
    pub text: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Chat {
    pub id: i64,
}

#[derive(Debug, Deserialize)]
pub struct User {
    pub id: i64,
}

pub fn extract_text_reply<'a>(update: &'a Update, user_id: i64) -> Option<&'a str> {
    let msg = update.message.as_ref()?;
    let from = msg.from.as_ref()?;

    // Only accept private-chat replies from the configured user.
    if from.id != user_id {
        return None;
    }
    if msg.chat.id != user_id {
        return None;
    }

    msg.text.as_deref()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_text_reply_filters_non_matching_user_or_chat() {
        let good = Update {
            update_id: 10,
            message: Some(Message {
                from: Some(User { id: 123 }),
                chat: Chat { id: 123 },
                text: Some("hi".to_string()),
            }),
        };

        assert_eq!(extract_text_reply(&good, 123), Some("hi"));
        assert_eq!(extract_text_reply(&good, 999), None);

        let wrong_chat = Update {
            update_id: 11,
            message: Some(Message {
                from: Some(User { id: 123 }),
                chat: Chat { id: 456 },
                text: Some("nope".to_string()),
            }),
        };
        assert_eq!(extract_text_reply(&wrong_chat, 123), None);

        let no_text = Update {
            update_id: 12,
            message: Some(Message {
                from: Some(User { id: 123 }),
                chat: Chat { id: 123 },
                text: None,
            }),
        };
        assert_eq!(extract_text_reply(&no_text, 123), None);
    }

    #[test]
    fn api_response_into_result_ok_requires_result() {
        let res = ApiResponse::<i64> {
            ok: true,
            result: None,
            description: None,
            error_code: None,
        };

        let err = res.into_result().unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("missing result despite ok=true"),
            "error was: {msg}"
        );
    }

    #[test]
    fn api_response_into_result_error_includes_code_and_description() {
        let res = ApiResponse::<i64> {
            ok: false,
            result: None,
            description: Some("nope".to_string()),
            error_code: Some(400),
        };

        let err = res.into_result().unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("telegram api error 400: nope"),
            "error was: {msg}"
        );
    }

    #[test]
    fn api_response_into_result_error_defaults_description_and_code() {
        let res = ApiResponse::<i64> {
            ok: false,
            result: None,
            description: None,
            error_code: None,
        };

        let err = res.into_result().unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("telegram api error 0: unknown telegram error"),
            "error was: {msg}"
        );
    }

    #[test]
    fn method_url_includes_base_url_token_and_method() {
        let mut client = TelegramClient::new("TOKEN".to_string());
        client.base_url = "https://example.test".to_string();

        assert_eq!(
            client.method_url("getUpdates"),
            "https://example.test/botTOKEN/getUpdates"
        );
    }
}
