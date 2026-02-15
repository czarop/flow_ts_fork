//! Test reading an FCS file to verify it parses correctly

use anyhow::Result;
use flow_fcs::Fcs;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <fcs_file>", args[0]);
        std::process::exit(1);
    }

    let fcs_path = &args[1];
    println!("📂 Opening FCS file: {}", fcs_path);

    // Open the FCS file
    let fcs = Fcs::open(fcs_path)?;

    println!("\n✅ File opened successfully!");
    println!("=========================");

    // Print some basic info
    println!("Version: {}", fcs.header.version);
    println!("Number of events: {}", fcs.get_event_count_from_dataframe());
    println!(
        "Number of parameters: {}",
        fcs.get_parameter_count_from_dataframe()
    );

    // Print parameter names
    println!("\nParameters:");
    let param_names = fcs.get_parameter_names_from_dataframe();
    for (idx, name) in param_names.iter().enumerate() {
        println!("  {}: {}", idx + 1, name);
    }

    // Check for any warnings in stderr (the parser prints warnings there)
    println!("\n✅ File parsed without errors!");

    Ok(())
}
