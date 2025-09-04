use crate::prelude::*;
use gql_client::{Client as GQLClient, ClientConfig};
use nestify::nest;
use serde::{Deserialize, Serialize};

pub struct CodSpeedAPIClient {
    gql_client: GQLClient,
}

fn build_gql_api_client(api_url: String, with_auth: bool) -> GQLClient {
    let headers = if with_auth {
        let mut headers = std::collections::HashMap::new();
        headers.insert(
            "Authorization".to_string(),
            std::env::var("CODSPEED_GRAPHQL_TOKEN").unwrap(),
        );
        headers
    } else {
        Default::default()
    };

    GQLClient::new_with_config(ClientConfig {
        endpoint: api_url,
        timeout: Some(10),
        headers: Some(headers),
        proxy: None,
    })
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FetchLocalRunReportVars {
    pub owner: String,
    pub name: String,
    pub run_id: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum ReportConclusion {
    AcknowledgedFailure,
    Failure,
    MissingBaseRun,
    Success,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchLocalRunReportRun {
    pub id: String,
    pub status: RunStatus,
    pub url: String,
    pub head_reports: Vec<FetchLocalRunReportHeadReport>,
    pub results: Vec<FetchLocalRunBenchmarkResult>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RunStatus {
    Completed,
    Failure,
    Pending,
    Processing,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchLocalRunReportHeadReport {
    pub id: String,
    pub impact: Option<f64>,
    pub conclusion: ReportConclusion,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FetchLocalRunBenchmarkResult {
    pub time: f64,
    pub benchmark: FetchLocalRunBenchmark,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FetchLocalRunBenchmark {
    pub name: String,
}

nest! {
    #[derive(Debug, Deserialize, Serialize)]*
    #[serde(rename_all = "camelCase")]*
    struct FetchLocalRunReportData {
        repository: pub struct FetchLocalRunReportRepository {
            run: FetchLocalRunReportRun,
        }
    }
}

#[derive(Debug)]
pub struct FetchLocalRunReportResponse {
    // pub allowed_regression: f64,
    pub run: FetchLocalRunReportRun,
}

impl CodSpeedAPIClient {
    pub fn new() -> CodSpeedAPIClient {
        let mut headers = std::collections::HashMap::new();
        headers.insert(
            "Authorization".to_string(),
            std::env::var("CODSPEED_GRAPHQL_TOKEN").unwrap(),
        );

        CodSpeedAPIClient {
            gql_client: GQLClient::new_with_config(ClientConfig {
                endpoint: "https://gql.codspeed.io/".to_owned(),
                timeout: Some(10),
                headers: Some(headers),
                proxy: None,
            }),
        }
    }

    pub async fn fetch_local_run_report(
        &self,
        vars: FetchLocalRunReportVars,
    ) -> Result<FetchLocalRunReportResponse> {
        let response = self
            .gql_client
            .query_with_vars_unwrap::<FetchLocalRunReportData, FetchLocalRunReportVars>(
                include_str!("queries/FetchLocalRunReport.gql"),
                vars.clone(),
            )
            .await;
        match response {
            Ok(response) => Ok(FetchLocalRunReportResponse {
                run: response.repository.run,
            }),
            Err(err) if err.contains_error_code("UNAUTHENTICATED") => {
                bail!("Your session has expired, please login again using `codspeed auth login`")
            }
            Err(err) => bail!("Failed to fetch local run report: {}", err),
        }
    }
}
