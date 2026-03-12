//! HTTP client implementation for the Typefully API.

use reqwest::Client;
use secrecy::{ExposeSecret, SecretString};
use tracing::debug;

use crate::error::ApiError;

use super::TypefullyApi;
use super::types::DraftListParams;

const BASE_URL: &str = "https://api.typefully.com/v2";

/// Concrete HTTP client for the Typefully API.
#[derive(Debug, Clone)]
pub struct TypefullyClient {
    client: Client,
    api_key: SecretString,
}

impl TypefullyClient {
    /// Create a new client with the given API key.
    #[must_use]
    pub fn new(api_key: SecretString) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    /// Create a client from an explicit key string (convenience for config init).
    #[must_use]
    pub fn from_key(key: &str) -> Self {
        Self::new(SecretString::from(key.to_string()))
    }

    async fn get_json(&self, path: &str) -> Result<serde_json::Value, ApiError> {
        let url = format!("{BASE_URL}{path}");
        debug!(method = "GET", %url, "API request");
        let resp = self
            .client
            .get(&url)
            .bearer_auth(self.api_key.expose_secret())
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    async fn post_json(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, ApiError> {
        let url = format!("{BASE_URL}{path}");
        debug!(method = "POST", %url, "API request");
        let resp = self
            .client
            .post(&url)
            .bearer_auth(self.api_key.expose_secret())
            .json(body)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    async fn patch_json(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, ApiError> {
        let url = format!("{BASE_URL}{path}");
        debug!(method = "PATCH", %url, "API request");
        let resp = self
            .client
            .patch(&url)
            .bearer_auth(self.api_key.expose_secret())
            .json(body)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    async fn delete_json(&self, path: &str) -> Result<serde_json::Value, ApiError> {
        let url = format!("{BASE_URL}{path}");
        debug!(method = "DELETE", %url, "API request");
        let resp = self
            .client
            .delete(&url)
            .bearer_auth(self.api_key.expose_secret())
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    async fn handle_response(resp: reqwest::Response) -> Result<serde_json::Value, ApiError> {
        let status = resp.status();

        if status == 429 {
            let retry = resp
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("unknown")
                .to_string();
            return Err(ApiError::RateLimited { retry_after: retry });
        }

        let text = resp.text().await?;

        if !status.is_success() {
            let message = serde_json::from_str::<serde_json::Value>(&text)
                .ok()
                .and_then(|v| v.get("error").and_then(|e| e.as_str()).map(String::from))
                .unwrap_or(text);
            return Err(ApiError::Response {
                status: status.as_u16(),
                message,
            });
        }

        if text.is_empty() {
            return Ok(serde_json::Value::Null);
        }

        Ok(serde_json::from_str(&text)?)
    }
}

#[async_trait::async_trait]
impl TypefullyApi for TypefullyClient {
    async fn get_me(&self) -> Result<serde_json::Value, ApiError> {
        self.get_json("/me").await
    }

    async fn get_social_sets(&self) -> Result<serde_json::Value, ApiError> {
        self.get_json("/social-sets").await
    }

    async fn list_drafts(
        &self,
        set_id: &str,
        params: &DraftListParams,
    ) -> Result<serde_json::Value, ApiError> {
        let qs = params.to_query_string();
        self.get_json(&format!("/social-sets/{set_id}/drafts?{qs}"))
            .await
    }

    async fn create_draft(
        &self,
        set_id: &str,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, ApiError> {
        self.post_json(&format!("/social-sets/{set_id}/drafts"), body)
            .await
    }

    async fn get_draft(&self, set_id: &str, draft_id: &str) -> Result<serde_json::Value, ApiError> {
        self.get_json(&format!("/social-sets/{set_id}/drafts/{draft_id}"))
            .await
    }

    async fn update_draft(
        &self,
        set_id: &str,
        draft_id: &str,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, ApiError> {
        self.patch_json(&format!("/social-sets/{set_id}/drafts/{draft_id}"), body)
            .await
    }

    async fn delete_draft(
        &self,
        set_id: &str,
        draft_id: &str,
    ) -> Result<serde_json::Value, ApiError> {
        self.delete_json(&format!("/social-sets/{set_id}/drafts/{draft_id}"))
            .await
    }

    async fn create_media_upload(
        &self,
        set_id: &str,
        file_name: &str,
        content_type: &str,
    ) -> Result<serde_json::Value, ApiError> {
        let body = serde_json::json!({
            "file_name": file_name,
            "content_type": content_type,
        });
        self.post_json(&format!("/social-sets/{set_id}/media/upload"), &body)
            .await
    }

    async fn upload_to_presigned(
        &self,
        url: &str,
        data: Vec<u8>,
        content_type: &str,
    ) -> Result<(), ApiError> {
        debug!(method = "PUT", %url, bytes = data.len(), "S3 upload");
        let resp = self
            .client
            .put(url)
            .header("Content-Type", content_type)
            .body(data)
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(ApiError::Response {
                status: status.as_u16(),
                message: text,
            });
        }
        Ok(())
    }

    async fn get_media_status(
        &self,
        set_id: &str,
        media_id: &str,
    ) -> Result<serde_json::Value, ApiError> {
        self.get_json(&format!("/social-sets/{set_id}/media/{media_id}"))
            .await
    }

    async fn list_tags(&self, set_id: &str) -> Result<serde_json::Value, ApiError> {
        self.get_json(&format!("/social-sets/{set_id}/tags")).await
    }

    async fn create_tag(&self, set_id: &str, name: &str) -> Result<serde_json::Value, ApiError> {
        let body = serde_json::json!({"name": name});
        self.post_json(&format!("/social-sets/{set_id}/tags"), &body)
            .await
    }
}
