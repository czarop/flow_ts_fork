//! Export mixing matrix to CSV for comparison with Julia
//!
//! This example creates a mixing matrix from single-stain controls and exports it to CSV.

use anyhow::Result;
use flow_fcs::Fcs;
use ndarray::Array2;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn export_matrix_to_csv(
    matrix: &Array2<f64>,
    path: &PathBuf,
    row_names: &[String],
    col_names: &[String],
) -> Result<()> {
    let mut file = File::create(path)?;

    // Write header: first column is row names, then column names
    write!(file, "RowName,")?;
    for (i, col_name) in col_names.iter().enumerate() {
        write!(file, "{}", col_name)?;
        if i < col_names.len() - 1 {
            write!(file, ",")?;
        }
    }
    writeln!(file)?;

    // Write data
    for (row_idx, row_name) in row_names.iter().enumerate() {
        write!(file, "{}", row_name)?;
        for col_idx in 0..matrix.ncols() {
            write!(file, ",{:.10e}", matrix[(row_idx, col_idx)])?;
        }
        writeln!(file)?;
    }

    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        eprintln!(
            "Usage: {} <unstained.fcs> <controls_dir> <output_matrix.csv>",
            args[0]
        );
        eprintln!("\nExample:");
        eprintln!("  {} unstained.fcs controls/ mixing_matrix.csv", args[0]);
        std::process::exit(1);
    }

    let unstained_path = &args[1];
    let controls_dir = PathBuf::from(&args[2]);
    let output_path = PathBuf::from(&args[3]);

    println!("📊 Exporting Mixing Matrix");
    println!("===========================");
    println!("Unstained control: {}", unstained_path);
    println!("Controls directory: {}", controls_dir.display());
    println!("Output: {}", output_path.display());

    // Load unstained control
    println!("\n📂 Loading unstained control...");
    let unstained_fcs = Fcs::open(unstained_path)?;

    // Use the CLI's unmix command logic to create the matrix
    // We'll need to call the internal function
    println!("\n⚠️  Note: This requires access to internal CLI functions.");
    println!("   For now, use the main CLI command to generate the matrix:");
    println!("\n   tru-ols unmix \\");
    println!("     --stained <any_sample.fcs> \\");
    println!("     --controls {} \\", unstained_path);
    println!("     --single-stain-controls {} \\", controls_dir.display());
    println!("     --output /tmp/temp.fcs");
    println!("\n   Then the mixing matrix will be created internally.");
    println!("\n   Alternatively, create mixing_matrix.csv manually with format:");
    println!("   RowName,Endmember1,Endmember2,...,Autofluorescence");
    println!("   Detector1,value,value,...,value");
    println!("   Detector2,value,value,...,value");
    println!("   ...");

    Ok(())
}
