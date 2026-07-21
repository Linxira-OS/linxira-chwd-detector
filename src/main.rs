// SPDX-License-Identifier: GPL-3.0-only

fn main() {
    if std::env::args_os().nth(1).is_some() {
        eprintln!("linxira-chwd-detector accepts no arguments");
        std::process::exit(2);
    }

    let report = linxira_chwd_detector::collect_and_detect();
    match serde_json::to_string_pretty(&report) {
        Ok(json) => println!("{json}"),
        Err(error) => {
            eprintln!("failed to serialize detection report: {error}");
            std::process::exit(1);
        }
    }
}
