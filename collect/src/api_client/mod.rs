use crate::prelude::*;
use gql_client::{Client as GQLClient, ClientConfig};
use nestify::nest;
use serde::{Deserialize, Serialize};

pub struct CodSpeedAPIClient {
    gql_client: GQLClient,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FetchRunReportVars {
    pub owner: String,
    pub name: String,
    pub run_id: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetLatestFinishedRunVars {
    pub owner: String,
    pub name: String,
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
pub struct FetchRunReportRun {
    pub id: String,
    pub status: RunStatus,
    pub url: String,
    pub results: Vec<FetchRunBenchmarkResult>,
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
pub struct FetchRunBenchmarkResult {
    pub time: f64,
    pub benchmark: FetchRunBenchmark,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FetchRunBenchmark {
    pub name: String,
    pub uri: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetLatestFinishedRun {
    pub id: String,
    pub date: String,
    pub status: RunStatus,
}

nest! {
    #[derive(Debug, Deserialize, Serialize)]*
    #[serde(rename_all = "camelCase")]*
    struct FetchRunReportData {
        repository: pub struct FetchRunReportRepository {
            run: FetchRunReportRun,
        }
    }
}

nest! {
    #[derive(Debug, Deserialize, Serialize)]*
    #[serde(rename_all = "camelCase")]*
    struct GetLatestFinishedRunData {
        repository: pub struct GetLatestFinishedRunRepository {
            runs: Vec<GetLatestFinishedRun>,
        }
    }
}

#[derive(Debug)]
pub struct FetchRunReportResponse {
    // pub allowed_regression: f64,
    pub run: FetchRunReportRun,
}

#[derive(Debug)]
pub struct GetLatestFinishedRunResponse {
    pub run: Option<GetLatestFinishedRun>,
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
        vars: FetchRunReportVars,
    ) -> Result<FetchRunReportResponse> {
        let response = self
            .gql_client
            .query_with_vars_unwrap::<FetchRunReportData, FetchRunReportVars>(
                include_str!("queries/FetchRunReport.gql"),
                vars.clone(),
            )
            .await;
        match response {
            Ok(response) => Ok(FetchRunReportResponse {
                run: response.repository.run,
            }),
            Err(err) if err.contains_error_code("UNAUTHENTICATED") => {
                bail!("Your session has expired, please login again using `codspeed auth login`")
            }
            Err(err) => bail!("Failed to fetch local run report: {}", err),
        }
    }

    pub async fn get_latest_finished_run(
        &self,
        vars: GetLatestFinishedRunVars,
    ) -> Result<GetLatestFinishedRunResponse> {
        let response = self
            .gql_client
            .query_with_vars_unwrap::<GetLatestFinishedRunData, GetLatestFinishedRunVars>(
                include_str!("queries/FetchLastRunId.gql"),
                vars.clone(),
            )
            .await;
        match response {
            Ok(response) => Ok(GetLatestFinishedRunResponse {
                run: response.repository.runs.into_iter().next(),
            }),
            Err(err) if err.contains_error_code("UNAUTHENTICATED") => {
                bail!("Your session has expired, please login again using `codspeed auth login`")
            }
            Err(err) => bail!("Failed to get latest finished run: {}", err),
        }
    }
}
