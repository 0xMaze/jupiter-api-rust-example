use anyhow::{anyhow, Result};
use quote::{QuoteRequest, QuoteResponse};
use reqwest::{Client, ClientBuilder, Response};
use serde::de::DeserializeOwned;
use swap::{SwapInstructionsResponse, SwapInstructionsResponseInternal, SwapRequest, SwapResponse};

pub mod quote;
mod route_plan_with_metadata;
mod serde_helpers;
pub mod swap;
pub mod transaction_config;

#[derive(Clone)]
pub struct JupiterSwapApiClient {
    pub base_path: String,
}

async fn check_is_success(response: Response) -> Result<Response> {
    if !response.status().is_success() {
        return Err(anyhow!(
            "Request status not ok: {}, body: {:?}",
            response.status(),
            response.text().await
        ));
    }
    Ok(response)
}

async fn check_status_code_and_deserialize<T: DeserializeOwned>(response: Response) -> Result<T> {
    check_is_success(response)
        .await?
        .json::<T>()
        .await
        .map_err(Into::into)
}

impl JupiterSwapApiClient {
    pub fn new(base_path: String) -> Self {
        Self { base_path }
    }

    fn build_client(proxy: Option<reqwest::Proxy>) -> Result<reqwest::Client> {
        match proxy {
            Some(proxy) => Ok(ClientBuilder::new().proxy(proxy).build()?),
            None => Ok(Client::new()),
        }
    }

    pub async fn quote(
        &self,
        quote_request: &QuoteRequest,
        proxy: Option<reqwest::Proxy>,
    ) -> Result<QuoteResponse> {
        let query = serde_qs::to_string(&quote_request)?;
        let client = Self::build_client(proxy)?;
        let response = client
            .get(format!("{}/quote?{query}", self.base_path))
            .send()
            .await?;
        check_status_code_and_deserialize(response).await
    }

    pub async fn swap(
        &self,
        swap_request: &SwapRequest,
        proxy: Option<reqwest::Proxy>,
    ) -> Result<SwapResponse> {
        let client = Self::build_client(proxy)?;
        let response = client
            .post(format!("{}/swap", self.base_path))
            .json(swap_request)
            .send()
            .await?;
        check_status_code_and_deserialize(response).await
    }

    pub async fn swap_instructions(
        &self,
        swap_request: &SwapRequest,
        proxy: Option<reqwest::Proxy>,
    ) -> Result<SwapInstructionsResponse> {
        let client = Self::build_client(proxy)?;
        let response = client
            .post(format!("{}/swap-instructions", self.base_path))
            .json(swap_request)
            .send()
            .await?;
        check_status_code_and_deserialize::<SwapInstructionsResponseInternal>(response)
            .await
            .map(Into::into)
    }
}
