//! API client and trait definitions for the Typefully API v2.

mod client;
mod types;

pub use client::TypefullyClient;
pub use types::*;

use crate::error::ApiError;

/// Trait for interacting with the Typefully API.
///
/// This trait enables mocking in tests by abstracting over the HTTP layer.
#[async_trait::async_trait]
pub trait TypefullyApi: Send + Sync {
    /// `GET /v2/me`
    async fn get_me(&self) -> std::result::Result<serde_json::Value, ApiError>;

    /// `GET /v2/social-sets`
    async fn get_social_sets(&self) -> std::result::Result<serde_json::Value, ApiError>;

    /// `GET /v2/social-sets/{set_id}/drafts`
    async fn list_drafts(
        &self,
        set_id: &str,
        params: &DraftListParams,
    ) -> std::result::Result<serde_json::Value, ApiError>;

    /// `POST /v2/social-sets/{set_id}/drafts`
    async fn create_draft(
        &self,
        set_id: &str,
        body: &serde_json::Value,
    ) -> std::result::Result<serde_json::Value, ApiError>;

    /// `GET /v2/social-sets/{set_id}/drafts/{draft_id}`
    async fn get_draft(
        &self,
        set_id: &str,
        draft_id: &str,
    ) -> std::result::Result<serde_json::Value, ApiError>;

    /// `PATCH /v2/social-sets/{set_id}/drafts/{draft_id}`
    async fn update_draft(
        &self,
        set_id: &str,
        draft_id: &str,
        body: &serde_json::Value,
    ) -> std::result::Result<serde_json::Value, ApiError>;

    /// `DELETE /v2/social-sets/{set_id}/drafts/{draft_id}`
    async fn delete_draft(
        &self,
        set_id: &str,
        draft_id: &str,
    ) -> std::result::Result<serde_json::Value, ApiError>;

    /// `POST /v2/social-sets/{set_id}/media/upload` (get presigned URL).
    async fn create_media_upload(
        &self,
        set_id: &str,
        file_name: &str,
        content_type: &str,
    ) -> std::result::Result<serde_json::Value, ApiError>;

    /// Upload bytes to a presigned S3 URL.
    async fn upload_to_presigned(
        &self,
        url: &str,
        data: Vec<u8>,
        content_type: &str,
    ) -> std::result::Result<(), ApiError>;

    /// `GET /v2/social-sets/{set_id}/media/{media_id}`
    async fn get_media_status(
        &self,
        set_id: &str,
        media_id: &str,
    ) -> std::result::Result<serde_json::Value, ApiError>;

    /// `GET /v2/social-sets/{set_id}/tags`
    async fn list_tags(
        &self,
        set_id: &str,
    ) -> std::result::Result<serde_json::Value, ApiError>;

    /// `POST /v2/social-sets/{set_id}/tags`
    async fn create_tag(
        &self,
        set_id: &str,
        name: &str,
    ) -> std::result::Result<serde_json::Value, ApiError>;
}
