use crate::infrastructure::Settings;
use crate::service::ServiceError;
use openidconnect::core::{CoreClient, CoreProviderMetadata};
use openidconnect::{
    ClientId, ClientSecret, EndpointMaybeSet, EndpointNotSet, EndpointSet, RedirectUrl,
};
use openidconnect::{IssuerUrl, reqwest};

pub type GoogleClient = CoreClient<
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointMaybeSet,
    EndpointMaybeSet,
>;

#[tracing::instrument(name = "auth.google.create_client", level = "info", skip_all)]
pub async fn create_core_client(
    settings: &Settings,
) -> Result<(GoogleClient, reqwest::Client), ServiceError> {
    let http_client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| {
            tracing::error!(
                provider = "google",
                step = "http_client",
                "OpenID client setup failed"
            );
            ServiceError::InternalError(e.to_string())
        })?;
    let issuer = IssuerUrl::new("https://accounts.google.com".into()).map_err(|e| {
        tracing::error!(
            provider = "google",
            step = "issuer_url",
            "OpenID client setup failed"
        );
        ServiceError::InternalError(e.to_string())
    })?;
    tracing::debug!(provider = "google", "starting OpenID provider discovery");
    let metadata = CoreProviderMetadata::discover_async(issuer, &http_client)
        .await
        .map_err(|e| {
            tracing::error!(
                provider = "google",
                step = "discovery",
                "OpenID client setup failed"
            );
            ServiceError::InternalError(e.to_string())
        })?;

    let callback = format!(
        "{}/auth/google/callback",
        settings.api_base_url.trim_end_matches("/")
    );

    let redirect_url = RedirectUrl::new(callback).map_err(|e| {
        tracing::error!(
            provider = "google",
            step = "callback_url",
            "OpenID client setup failed"
        );
        ServiceError::InternalError(e.to_string())
    })?;

    let client = CoreClient::from_provider_metadata(
        metadata,
        ClientId::new(settings.client_id.clone()),
        Some(ClientSecret::new(settings.client_secret.clone())),
    )
    .set_redirect_uri(redirect_url);
    tracing::info!(provider = "google", "OpenID client ready");
    Ok((client, http_client))
}
