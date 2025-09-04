#[tokio::main]
async fn main() -> anyhow::Result<()> {
    collect_bench_results::run().await?;

    Ok(())
}
