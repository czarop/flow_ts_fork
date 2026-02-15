//! Create a mixing matrix CSV from single-stain controls
//!
//! This example creates a mixing matrix from single-stain controls and exports it to CSV
//! for use with the comparison example.

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
        eprintln!("\nThis will create a mixing matrix CSV from single-stain controls.");
        std::process::exit(1);
    }

    let unstained_path = &args[1];
    let controls_dir = PathBuf::from(&args[2]);
    let output_path = PathBuf::from(&args[3]);

    println!("📊 Creating Mixing Matrix CSV");
    println!("==============================");
    println!("Unstained control: {}", unstained_path);
    println!("Controls directory: {}", controls_dir.display());
    println!("Output: {}", output_path.display());

    // Load unstained control
    println!("\n📂 Loading unstained control...");
    let unstained_fcs = Fcs::open(unstained_path)?;

    // Use the CLI command to create the matrix
    // Since we can't easily access internal functions from examples,
    // we'll provide instructions to use the main CLI
    println!("\n💡 To create the mixing matrix CSV:");
    println!("\n   1. Run the main CLI command (it creates the matrix internally):");
    println!(
        "      tru-ols unmix --stained <any_sample.fcs> --controls {} --single-stain-controls {} --output /tmp/temp.fcs",
        unstained_path,
        controls_dir.display()
    );
    println!("\n   2. The mixing matrix is created internally but not yet exported.");
    println!("\n   3. For now, you can:");
    println!("      - Create the CSV manually (see format below)");
    println!("      - Or wait for export functionality to be added");
    println!("\n📝 Manual CSV format:");
    println!("   RowName,Endmember1,Endmember2,...,Autofluorescence");
    println!("   Detector1,value,value,...,value");
    println!("   Detector2,value,value,...,value");
    println!("   ...");
    println!("\n   Where values are normalized spectral signatures (0.0 to 1.0)");

    Ok(())
}
