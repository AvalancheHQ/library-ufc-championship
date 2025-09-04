mod prelude;

mod api_client;

use crate::api_client::CodSpeedAPIClient;
use crate::api_client::{FetchRunReportVars, GetLatestFinishedRunVars};
use crate::prelude::*;

pub async fn run() -> Result<()> {
    let client = CodSpeedAPIClient::new();

    let latest_run_response = client
        .get_latest_finished_run(GetLatestFinishedRunVars {
            owner: "AvalancheHQ".to_string(),
            name: "library-ufc-championship".to_string(),
        })
        .await?;

    let run_id = match latest_run_response.run {
        Some(run) => run.id,
        None => bail!("No finished runs found for this repository"),
    };

    let run = client
        .fetch_local_run_report(FetchRunReportVars {
            owner: "AvalancheHQ".to_string(),
            name: "library-ufc-championship".to_string(),
            run_id,
        })
        .await?
        .run;

    dbg!(run);

    Ok(())
}
