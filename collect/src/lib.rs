mod prelude;

mod api_client;

use crate::api_client::CodSpeedAPIClient;
use crate::api_client::{FetchRunBenchmarkResult, FetchRunReportVars, GetLatestFinishedRunVars};
use crate::prelude::*;
use std::collections::HashMap;
use std::fs;

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

    let markdown = generate_markdown_recap(&run.results);

    fs::write("./results.md", &markdown)?;
    println!("Results written to ./results.md");

    Ok(())
}

#[derive(Debug, Clone)]
struct BenchmarkEntry {
    category: String,
    usecase: String,
    test_function: String,
    library: String,
    time_seconds: f64,
}

#[derive(Debug)]
struct UsecaseTable {
    usecase: String,
    libraries: Vec<String>,
    test_functions: Vec<String>,
    results: HashMap<String, HashMap<String, f64>>, // test_function -> library -> time_ms
}

#[derive(Debug)]
struct CategorySection {
    category: String,
    tables: Vec<UsecaseTable>,
}

fn parse_benchmark_results(results: &[FetchRunBenchmarkResult]) -> Vec<BenchmarkEntry> {
    let mut entries = Vec::new();

    for result in results {
        let parts: Vec<&str> = result.benchmark.uri.split("::").collect();
        if parts.len() < 3 {
            continue;
        }

        let category_path = parts[0];
        let test_function = parts[1];
        let library = parts[2];

        // Extract category and usecase from the path
        let path_parts: Vec<&str> = category_path.split('/').collect();
        let category = path_parts[0].to_string();
        let usecase = if path_parts.len() > 1 {
            path_parts[1].replace(".rs", "")
        } else {
            "default".to_string()
        };

        entries.push(BenchmarkEntry {
            category,
            usecase,
            test_function: test_function.to_string(),
            library: library.to_string(),
            time_seconds: result.time,
        });
    }

    entries
}

type CategoryMap = HashMap<String, HashMap<String, HashMap<String, HashMap<String, f64>>>>;

fn format_time_human_readable(seconds: f64) -> String {
    let (value, unit) = if seconds >= 1.0 {
        (seconds, "s")
    } else if seconds >= 1e-3 {
        (seconds * 1e3, "ms")
    } else if seconds >= 1e-6 {
        (seconds * 1e6, "μs")
    } else {
        (seconds * 1e9, "ns")
    };

    // Format with 3 significant digits, removing trailing zeros
    let formatted = format!("{:.3}", value);
    let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
    format!("{}{}", trimmed, unit)
}

fn organize_into_sections(entries: Vec<BenchmarkEntry>) -> Vec<CategorySection> {
    let mut categories: CategoryMap = HashMap::new();

    // Group by category -> usecase -> test_function -> library -> time
    for entry in entries {
        categories
            .entry(entry.category)
            .or_default()
            .entry(entry.usecase)
            .or_default()
            .entry(entry.test_function)
            .or_default()
            .insert(entry.library, entry.time_seconds);
    }

    let mut sections = Vec::new();

    for (category, usecases) in categories {
        let mut tables = Vec::new();

        for (usecase, tests) in usecases {
            // Collect all unique libraries and test functions for this usecase
            let mut all_libraries: std::collections::BTreeSet<String> =
                std::collections::BTreeSet::new();
            let mut all_test_functions: std::collections::BTreeSet<String> =
                std::collections::BTreeSet::new();

            for (test_function, libraries) in &tests {
                all_test_functions.insert(test_function.clone());
                for library in libraries.keys() {
                    all_libraries.insert(library.clone());
                }
            }

            tables.push(UsecaseTable {
                usecase,
                libraries: all_libraries.into_iter().collect(),
                test_functions: all_test_functions.into_iter().collect(),
                results: tests,
            });
        }

        sections.push(CategorySection { category, tables });
    }

    sections
}

fn generate_markdown_from_sections(sections: Vec<CategorySection>) -> String {
    let mut output = String::new();

    for section in sections {
        output.push_str(&format!("## {}\n\n", section.category));

        for table in section.tables {
            output.push_str(&format!("### {}\n\n", table.usecase));

            // Create table header
            output.push_str("| Test |");
            for library in &table.libraries {
                output.push_str(&format!(" {} |", library));
            }
            output.push_str("\n|------|");
            for _ in &table.libraries {
                output.push_str("------|");
            }
            output.push('\n');

            // Create table rows
            for test_function in &table.test_functions {
                output.push_str(&format!("| {} |", test_function));
                for library in &table.libraries {
                    if let Some(time) = table
                        .results
                        .get(test_function)
                        .and_then(|libs| libs.get(library))
                    {
                        output.push_str(&format!(" {} |", format_time_human_readable(*time)));
                    } else {
                        output.push_str(" - |");
                    }
                }
                output.push('\n');
            }

            output.push('\n');
        }
    }

    output
}

fn generate_markdown_recap(results: &[FetchRunBenchmarkResult]) -> String {
    let entries = parse_benchmark_results(results);
    let sections = organize_into_sections(entries);
    generate_markdown_from_sections(sections)
}
