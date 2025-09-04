mod prelude;

mod api_client;

use crate::api_client::CodSpeedAPIClient;
use crate::api_client::FetchLocalRunReportVars;
use crate::prelude::*;

pub async fn run() -> Result<()> {
    let client = CodSpeedAPIClient::new();

    let run = client
        .fetch_local_run_report(FetchLocalRunReportVars {
            owner: "AvalancheHQ".to_string(),
            name: "library-ufc-championship".to_string(),
            run_id: "68b9771810e5c6feb2f6ef4f".to_string(),
        })
        .await?
        .run;

    dbg!(run);

    Ok(())
}
