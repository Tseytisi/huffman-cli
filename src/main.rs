mod argparse;
mod tree_format;

use argparse::{Commands, Args};
use huffman::tree::HuffmanTree;
use crate::argparse::{DecodeArgs, EncodeArgs, EstimateArgs, TreeArgs, TreeExportFormat};
use crate::tree_format::ascii_tree_formatter;
use huffman;

use clap::Parser;

use std::collections::HashMap;
use std::{fs, io};
use std::io::Write;
use std::path::PathBuf;
use huffman::file::{HuffmanFile, PlainFile};

fn main() {
    // main_testing();
    let args = argparse::Args::parse();
    match &args.command {
        Commands::Encode(encode_args) => handle_encoding(&args, encode_args),
        Commands::Decode(decode_args) => handle_decoding(&args, decode_args),
        Commands::Estimate(est_args) => handle_estimate(&args, est_args),
        Commands::Tree(tree_args) => handle_tree_command(&args, tree_args),
    }
}

fn handle_encoding(args: &Args, encode_args: &EncodeArgs) {
    if args.debug && !args.silent {
        println!("Using 'encode' with these arguments: {:#?}", args);
    }

    // Read data from the file on disk, or exit if there was an issue
    let file_data = read_data_from_file(&encode_args.input, "input file", args.debug);
    let mut file = PlainFile::new(file_data);

    // Set the chunk size
    if encode_args.use_best_compression {
        // If we use the best compression
        file.set_chunk_size(file.predict_optimal_chunk_size());
        if !args.silent {
            println!("Using chunk size: {} bits", file.get_chunk_size());
        }
    } else {
        file.set_chunk_size(encode_args.chunk_size);

        if !file.has_valid_chunk_size() {
            println!("Error: {} bits is not a valid chunk size for this file", encode_args.chunk_size);
            std::process::exit(1);
        }
    }

    if !args.silent {
        print!("Building encoding tree... ");
        io::stdout().flush().expect("Error writing to console");
    }
    match file.build_tree() {
        Ok(_) => if !args.silent { println!("DONE"); },
        Err(e) => {
            if !args.silent { println!("FAILED"); }
            println!("Error: Failed to build encoding tree");
            if !args.silent && args.debug {
                println!("Debug > The following error occurred while building the tree: {}", e);
            }
        }
    }

    if !args.silent {
        print!("Encoding... ");
        io::stdout().flush().expect("Error writing to console");
    }
    match file.encode() {
        Ok(_) => if !args.silent { println!("DONE"); },
        Err(e) => {
            if !args.silent { println!("FAILED"); }
            println!("Error: Failed to encode file");
            if !args.silent && args.debug {
                println!("Debug > The following error occurred while encoding: {}", e);
            }
        }
    }

    let output_path = if let Some(custom_path) = &encode_args.output {
        PathBuf::from(custom_path)
    } else {
        let input_file_name_copy = PathBuf::from(&encode_args.input);
        let mut new_output_file_name = input_file_name_copy.into_os_string();
        // Add the .dat extension to the input's filename
        new_output_file_name.push(".dat");
        PathBuf::from(new_output_file_name)
    };

    if let Some(tree_path) = &encode_args.tree_path {
        // If a tree path has been defined, we export the tree separately

        // Data
        if !args.silent {
            print!("Exporting data... ");
            io::stdout().flush().expect("Error writing to console");
        }
        match file.export_data_only(&output_path) {
            Ok(_) => if !args.silent {
                println!("DONE");
                println!("Encoded data written to '{}'", &output_path.display());
            },
            Err(e) => {
                if !args.silent { println!("FAILED"); }
                println!("Error: Failed to export data to file at '{}'", &output_path.display());
                if !args.silent && args.debug {
                    println!("Debug > The following error occurred while writing to disk: {}", e);
                }
            }
        }
        // Tree
        if !args.silent {
            print!("Exporting tree... ");
            io::stdout().flush().expect("Error writing to console");
        }
        match file.export_tree_only(&tree_path) {
            Ok(_) => if !args.silent {
                println!("DONE");
                println!("Encoding tree written to '{}'", &tree_path.display());
            },
            Err(e) => {
                if !args.silent { println!("FAILED"); }
                println!("Error: Failed to export encoding tree to file at '{}'", &tree_path.display());
                if !args.silent && args.debug {
                    println!("Debug > The following error occurred while writing to disk: {}", e);
                }
            }
        }
    } else {
        // If no tree path has been defined, we write the data to one file
        if !args.silent {
            print!("Exporting encoded file... ");
            io::stdout().flush().expect("Error writing to console");
        }
        match file.export_file(&output_path) {
            Ok(_) => if !args.silent {
                println!("DONE");
                println!("Encoded file written to '{}'", &output_path.display());
            },
            Err(e) => {
                if !args.silent { println!("FAILED"); }
                println!("Error: Failed to export to file at '{}'", &output_path.display());
                if !args.silent && args.debug {
                    println!("Debug > The following error occurred while writing to disk: {}", e);
                }
            }
        }
    }

    // If set, print statistics
    if !args.silent && encode_args.print_statistics {
        if let Some(stats) = file.generate_statistics() {
            println!("Encoding statistics for '{}':", &encode_args.input.display());
            println!(" > Original file total size:      {:5} bytes", &stats.decoded_file_size);
            println!(" >> Original file chunk size:     {:5} bits", &stats.chunk_size);
            println!(" >> Data chunks in original file: {:5} chunks", &stats.chunk_count);
            println!();
            println!(" > Encoded file total size:       {:5} bytes", &stats.encoded_file_size);
            println!(" >> Encoded file data size:       {:5} bytes", &stats.encoded_data_size);
            println!(" >> Encoding tree data size:      {:5} bytes", &stats.serialised_tree_size);
            println!(" >> Encoding tree depth:          {:5}", &stats.encoding_tree_depth);
            println!();
            println!(" > Data compression ratio:        {:5}", &stats.data_compression_ratio);
            println!(" > File compression ratio:        {:5}", &stats.file_compression_ratio);
            println!();
            println!(" > Covered values ratio:          {:5}", &stats.covered_values_ratio);
            println!(" >> Number of unique values:      {:5}", &stats.unique_values);
            println!(" >> Number of possible values:    {:5}", 2u64.pow(stats.chunk_size));
            println!(" > Encoded data offset:           {:5} bytes", &stats.encoded_data_offset);
        } else {
            println!("Failed to generate statistics");
        }
    }
}

fn handle_decoding(args: &Args, decode_args: &DecodeArgs) {
    if args.debug && !args.silent {
        println!("Using 'decode' with these arguments: {:#?}", args);
    }

    // Read data from the file on disk, or exit if there was an issue
    let file_data: Vec<u8> = read_data_from_file(&decode_args.input, "input file", args.debug);
    if !args.silent {
        print!("Importing data... ");
        io::stdout().flush().expect("Error writing to console");
    }
    let mut file: HuffmanFile;
    if let Some(tree_path) = &decode_args.tree_path {
        // If a tree was defined, load it or exit
        let tree_data = read_data_from_file(tree_path, "tree file", args.debug);

        match HuffmanFile::from_raw_data(file_data, tree_data) {
            Ok(new_file) => file = new_file,
            Err(e) => {
                if !args.silent { println!("FAILED"); }
                println!("Error: Could not build tree from the given tree file");
                if args.debug && !args.silent {
                    println!("Debug > Error while building the tree: {}", e);
                }
                std::process::exit(1);
            }
        }
    } else {
        // If the input was one file, read the data as one file
        match HuffmanFile::new(file_data) {
            Ok(new_file) => file = new_file,
            Err(e) => {
                if !args.silent { println!("FAILED"); }
                println!("Error: Data in file '{}' is not valid encoded data that this program can read",
                         &decode_args.input.display());
                if args.debug && !args.silent {
                    println!("Debug > Error while creating new file from data: {}", e);
                }
                std::process::exit(1);
            }
        }
    }
    if !args.silent {
        println!("DONE");
        print!("Decoding... ");
        io::stdout().flush().expect("Error writing to console");
    }

    // Decode the file
    match file.decode() {
        Ok(_) => println!("DONE"),
        Err(e) => {
            println!("FAILED");
            println!("Error: Could not decode file");
            if !args.silent && args.debug {
                println!("Debug > The following error occurred while decoding: {}", e);
            }
            std::process::exit(1);
        }
    }

    // Define the output path and filename
    let output_path: PathBuf = if let Some(custom_path) = &decode_args.output {
        // If an output file has been defined
        PathBuf::from(custom_path)
    } else {
        // Remove whatever extension is on the file as the output file
        let mut new_path = PathBuf::from(&decode_args.input);
        new_path.set_extension("");
        new_path
    };

    if !args.silent {
        print!("Writing decoded file to disk... ");
        io::stdout().flush().expect("Error writing to console");
    }

    // Try to write the output data to disk
    if let Some(output_data) = file.get_output() {
        match fs::write(&output_path, output_data) {
            Ok(_) => if !args.silent {
                println!("DONE");
                println!("Decoded file written to '{}'", &output_path.display());
            },
            Err(e) => {
                if !args.silent {
                    println!("FAILED");
                }
                println!("Error: Could not write output data to disk at '{}'", &output_path.display());
                if !args.silent && args.debug {
                    println!("Debug > Error from fs::write : {}", e);
                }
            }
        }
    }

    // If set, print statistics
    if !args.silent && decode_args.print_statistics {
        if let Some(stats) = file.generate_statistics() {
            println!("Decoding statistics for '{}':", &decode_args.input.display());
            println!(" > Original file total size:      {:5} bytes", &stats.decoded_file_size);
            println!(" >> Original file chunk size:     {:5} bits", &stats.chunk_size);
            println!(" >> Data chunks in original file: {:5} chunks", &stats.chunk_count);
            println!();
            println!(" > Encoded file total size:       {:5} bytes", &stats.encoded_file_size);
            println!(" >> Encoded file data size:       {:5} bytes", &stats.encoded_data_size);
            println!(" >> Encoding tree data size:      {:5} bytes", &stats.serialised_tree_size);
            println!(" >> Encoding tree depth:          {:5}", &stats.encoding_tree_depth);
            println!();
            println!(" > Data compression ratio:        {:5}", &stats.data_compression_ratio);
            println!(" > File compression ratio:        {:5}", &stats.file_compression_ratio);
            println!();
            println!(" > Covered values ratio:          {:5}", &stats.covered_values_ratio);
            println!(" >> Number of unique values:      {:5}", &stats.unique_values);
            println!(" >> Number of possible values:    {:5}", 2u64.pow(stats.chunk_size));
            println!(" > Encoded data offset:           {:5} bytes", &stats.encoded_data_offset);
        } else {
            println!("Failed to generate statistics");
        }
    }
}

fn handle_estimate(args: &Args, estimate_args: &EstimateArgs) {
    if args.debug && !args.silent {
        println!("Using 'estimate' with these arguments: {:#?}", args);
    }
    let file_data: Vec<u8> = read_data_from_file(&estimate_args.input, "input file", args.debug);
    // Technically, in silent mode the file not found error would still pop up, but the rest should not be printed
    if !args.silent {
        let file = huffman::file::PlainFile::new(file_data);
        let data_size = file.get_input().len() * 8;
        let mut divisors = Vec::new();
        for i in 2..=64 {
            if data_size % i == 0 {
                divisors.push(i);
            }
        }
        print!("Estimating file sizes for chunk sizes: ");
        for i in 0..divisors.len() {
            if i == 0 {
                print!("{}", divisors.get(0).unwrap());
            } else if i < divisors.len() - 1 {
                print!(", {}", divisors.get(i).unwrap());
            } else if i == divisors.len() - 1 {
                println!(" and {}", divisors.get(i).unwrap());
            }
        }

        let estimates = file.predict_file_sizes();
        if estimates.is_empty() {
            println!("Error: Could not estimate encoded file sizes for '{}'", &estimate_args.input.display());
        } else {
            println!("Encoded file sizes for '{}' ({} bytes)", &estimate_args.input.display(), file.get_input().len());
            println!("Bit# | Data size (bytes) | Tree size (bytes) | Total (bytes)");
            for est in estimates {
                println!("  {:2} | {:17} | {:17} | {:13}", est.chunk_size, est.file_size, est.tree_size, est.total_size);
            }
        }
    }
}

fn handle_tree_command(args: &Args, tree_args: &TreeArgs) {
    if tree_args.counts && !tree_args.print_table {
        println!("Warning: --counts has no effect without --table");
    }

    // Read data from the file on disk, or exit if there was an issue
    let file_data = read_data_from_file(&tree_args.input, "input file", args.debug);
    let mut file = PlainFile::new(file_data);

    file.set_chunk_size(tree_args.chunk_size);

    if !file.has_valid_chunk_size() {
        println!("Error: {} bits is not a valid chunk size for this file", tree_args.chunk_size);
        std::process::exit(1);
    }

    if !args.silent {
        print!("Building encoding tree... ");
        io::stdout().flush().expect("Error writing to console");
    }
    match file.build_tree() {
        Ok(_) => if !args.silent { println!("DONE"); },
        Err(e) => {
            if !args.silent { println!("FAILED"); }
            println!("Error: Failed to build encoding tree");
            if !args.silent && args.debug {
                println!("Debug > The following error occurred while building the tree: {}", e);
            }
            std::process::exit(1);
        }
    }

    let tree: &HuffmanTree<u64> = file.get_tree().unwrap();

    let mut occurrence_map: Option<HashMap<u64, u32>> = None;
    if tree_args.tree_path.is_some() || tree_args.print_table {
        // Both need a formatter
        let mut formatter: fn(&u64) -> String;
        let formatter_type: TreeExportFormat;
        if let Some(format) = &tree_args.format {
            formatter = match format {
                TreeExportFormat::Ascii => {
                    if tree_args.chunk_size <= 8 {
                        tree_format::ascii_tree_formatter::<true>
                    } else {
                        println!("ASCII formatter cannot be used for chunk sizes above 8 bits");
                        std::process::exit(1);
                    }
                },
                TreeExportFormat::Hex => tree_format::initialised_hex_tree_formatter(tree_args.chunk_size),
                TreeExportFormat::SmallHex => |x: &u64| format!("{:x}", x),
                TreeExportFormat::Dec => |t: &u64| t.to_string(),
                TreeExportFormat::Bin => tree_format::initialised_bin_tree_formatter(tree_args.chunk_size),
                TreeExportFormat::SmallBin => |t: &u64| format!("{:b}", t),
            };
            formatter_type = *format;
        } else {
            if tree_args.chunk_size == 8 {
                formatter = tree_format::ascii_tree_formatter::<true>;
                formatter_type = TreeExportFormat::Ascii;
            } else if tree_args.chunk_size % 4 == 0 {
                formatter = tree_format::initialised_hex_tree_formatter(tree_args.chunk_size);
                formatter_type = TreeExportFormat::Hex;
            } else {
                formatter = |t: &u64| t.to_string();
                formatter_type = TreeExportFormat::Dec;
            }
        }

        if let Some(output_path) = &tree_args.tree_path {
            let file_content = tree.export_tree_dot(&formatter);
            match std::fs::write(output_path, file_content) {
                Ok(_) => println!("Tree exported to {output_path:?}"),
                Err(e) => println!("Failed to export tree: {e}"),
            }
        }


        if tree_args.print_table {
            let mut values = tree.get_values();

            let strlen = match formatter_type {
                TreeExportFormat::Ascii => 1,
                TreeExportFormat::Hex => ((tree_args.chunk_size as usize - 1) / 4) + 3,
                TreeExportFormat::SmallHex => ((tree_args.chunk_size as usize - 1) / 4) + 1,
                TreeExportFormat::Dec => 2u32.pow(tree_args.chunk_size).ilog10() as usize + 1,
                TreeExportFormat::Bin => tree_args.chunk_size as usize + 2,
                TreeExportFormat::SmallBin => tree_args.chunk_size as usize,
            };

            // Switch to un-escaped ASCII formatter for the terminal
            if formatter_type == TreeExportFormat::Ascii {
                formatter = ascii_tree_formatter::<false>
            }

            let column_width = 5.max(strlen);
            println!("\nEncoding tree content:");
            if tree_args.counts {
                occurrence_map = match file.calculate_occurrences() {
                    Ok(x) => Some(x),
                    Err(s) => {
                        // This has already happened once, why would it fail now?
                        println!("Failed to calculate value occurrences: {s}");
                        std::process::exit(1);
                    }
                };

                // Sort but include occurrence counts
                if let Some(ocm) = occurrence_map {
                    values.sort_by(
                        |(value_a, (path_a, level_a)), (value_b, (path_b, level_b))|
                        if level_a != level_b {
                            level_a.cmp(level_b)
                        } else if let Some(v_b) = ocm.get(*value_b) {
                            // Reverse this comparison (highest first)
                            match v_b.cmp(ocm.get(*value_a).unwrap()) {
                                std::cmp::Ordering::Equal => {
                                    path_a.cmp(path_b)
                                },
                                x => x
                            }
                        } else {
                            panic!("Value missing from occurrence map");
                        }
                    );
                    // Put it back put it back put it back
                    occurrence_map = Some(ocm);
                }
            } else {
                occurrence_map = Some(HashMap::new());
                values.sort_by(
                    |(_, (path_a, level_a)), (_, (path_b, level_b))|
                    if level_a != level_b {
                        level_a.cmp(level_b)
                    } else {
                        path_a.cmp(path_b)
                    }
                );
            }

            let path_column_width: usize = 4.max(tree.calculate_depth() as usize);
            // Cutely borrows your occurrence map
            if let Some(ocm) = occurrence_map {
                print!("| {:column_width$} | Path length |", "Value");
                if tree_args.paths {
                    print!(" {:path_column_width$} |", "Path");
                }
                if tree_args.counts {
                    print!(" Occurrences |");
                }
                println!();
                for (value, (path, level)) in values {
                    print!("| {:column_width$} | {:11} |", formatter(&value), level);
                    if tree_args.paths {
                        print!(" {:path_column_width$} |", format!("{:0lvl$b}", path, lvl=level as usize));
                    }
                    if tree_args.counts {
                        print!(" {:11} |", ocm.get(value).unwrap());
                    }
                    println!();
                }
                // Put it back put it back put it back
                occurrence_map = Some(ocm);
            }
        }
    }

    let stats = if tree_args.counts {
        file.generate_pre_encoding_statistics(Some(&occurrence_map.unwrap())).unwrap()
    } else {
        file.generate_pre_encoding_statistics(None).unwrap()
    };

    println!("\nStatistics for file {:?} @ chunk size {}b:", tree_args.input, stats.chunk_size);
    println!("  Original file size:");
    println!("    Bytes:       {}", stats.decoded_file_size);
    if tree_args.chunk_size != 8 {
        println!("    Chunks:      {}", stats.chunk_count);
    }
    println!("  Encoded file size:");
    println!("    Total:       {} bytes", stats.encoded_file_size);
    println!("      Data:      {} bytes", stats.encoded_data_size);
    println!("      Tree:      {} bytes", stats.serialised_tree_size);
    println!("  Data compression ratio: {:.4}", stats.data_compression_ratio);
    println!("  File compression ratio: {:.4}", stats.file_compression_ratio);
    println!("  Tree depth:    {} levels", stats.encoding_tree_depth);
    println!("  Unique chunks: {} / {}", stats.unique_values, 2f32.powi(stats.chunk_size as i32));
    println!("    Ratio: {:.2}%", stats.covered_values_ratio * 100.0);
}

fn read_data_from_file(filepath: &PathBuf, filetype: &str, debug: bool) -> Vec<u8> {
    match fs::read(PathBuf::from(filepath)) {
        Ok(file) => return file,
        Err(e) => {
            match e.kind() {
                io::ErrorKind::NotFound => {
                    println!("Error: {} not found at '{}'", filetype, &filepath.display());
                }
                _ => {
                    println!("Error: Could not read {} at '{}'", filetype, &filepath.display());
                }
            }
            if debug {
                println!("Debug > Error message: {}", e);
            }
            std::process::exit(1);
        }
    }
}
