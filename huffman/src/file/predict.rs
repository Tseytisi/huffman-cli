use crate::calculate::{calculate_levels, count_bytes};
use crate::file::PlainFile;

use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use crate::FILE_HEADER;

/// File size estimation of the encoded file, in bytes.
#[derive(Debug, Clone)]
pub struct Prediction {
    pub chunk_size: u32,
    pub tree_size: u32,
    pub file_size: u64,
    pub total_size: u64,
}

impl PlainFile {
    /// Calculates the predicted file sizes for all possible chunk size settings and returns the
    /// chunk size that results in the smallest total file size.
    ///
    /// Warning: This is a very expensive function. For files that have a lot of different possible
    /// chunk sizes, this can easily take longer than the full encoding process for any one chunk
    /// size.
    pub fn predict_optimal_chunk_size(&self) -> u32 {
        let predictions = self.predict_file_sizes();
        if predictions.is_empty() {
            panic!("Received no predictions - This should not be possible");
        }

        let mut smallest_file_size = u64::MAX;
        let mut smallest_chunk_size = 0;
        for p in predictions {
            if p.total_size < smallest_file_size {
                smallest_file_size = p.total_size;
                smallest_chunk_size = p.chunk_size;
            }
        }
        return smallest_chunk_size;
    }

    /// Calculates the resulting file sizes for every possible chunk size setting.
    /// Basically runs [PlainFile::predict_file_size_for] on multiple threads simultaneously.
    pub fn predict_file_sizes(&self) -> Vec<Prediction> {
        // Multithread benchmark (same file and parameters):
        // Single threaded: 79 seconds (CPU time 79 seconds)
        // Multi threaded: 27 seconds (CPU time 106 seconds)

        let data_size = self.get_input().len() * 8;
        let mut divisors = Vec::new();
        for i in 2..=64 {
            if data_size % i == 0 {
                divisors.push(i);
            }
        }

        let mut predictions: Vec<Prediction> = Vec::new();

        // Single threaded variant
        // for divisor in divisors {
        //     println!("Running for bit size {}", divisor);
        //     if let Ok(est) = self.predict_file_size_for(divisor as u32) {
        //         predictions.push(est);
        //     }
        // }

        // Thread vector
        let mut thrector = Vec::new();

        // We clone the data once in total
        let data_copy = self.input_data.clone();
        // Then we create a thread-safe reference counter so we can share the data among threads
        let data_ref = Arc::new(data_copy);

        for divisor in divisors {
            // Create a clone of the reference, because the thread moves this reference and we
            // wouldn't be able to use it for the next thread
            let thread_data = Arc::clone(&data_ref);
            thrector.push(thread::spawn(move || predict_size_for_data(&thread_data, divisor as u32)));
        }

        // Wait for each thread, in order, and push the result to the output vector when it's ready
        for t in thrector {
            if let Ok(thread_result) = t.join() {
                if let Ok(func_result) = thread_result {
                    predictions.push(func_result.clone());
                }
            }
        }

        predictions
    }

    /// Calculate the file size of the encoded file if the input were to be encoded with the given
    /// chunk size.
    ///
    /// Warning: This is an expensive function. For large chunk sizes, this function can easily take
    /// more time to run than encoding the file.
    pub fn predict_file_size_for(&self, chunk_size: u32) -> Result<Prediction, String> {
        return predict_size_for_data(&self.input_data, chunk_size);
    }
}

// Moved this function out of the 'impl' to make it easier to multithread
fn predict_size_for_data(data: &Vec<u8>, chunk_size: u32) -> Result<Prediction, String> {
    // println!("Running 'count_bytes' for bit size {}", chunk_size);
    return if let Ok(count_map) = count_bytes(data, chunk_size as u32) {
        // println!("Running calculate_levels for bit size {}", chunk_size);
        let level_map = calculate_levels(count_map.clone());

        // Calculate output file size
        let mut file_size = 0;
        for (value, path_length) in &level_map {
            if let Some(occurrences) = count_map.get(value) {
                file_size += (occurrences * path_length) as u64;
            }
        }
        // File size in bytes
        file_size = (file_size as f64 / 8f64).ceil() as u64;

        // Calculate tree size
        // Space taken up by the values
        let mut tree_size = 0u32;
        tree_size += count_map.len() as u32 * chunk_size;
        let mut depth = 0;
        let mut nodes_per_level = HashMap::new();
        for (_, v) in level_map {
            if v > depth {
                depth = v;
            }
            if let Some(count) = nodes_per_level.get_mut(&v) {
                *count += 1;
            } else {
                nodes_per_level.insert(v, 1u32);
            }
        }
        // Space taken up by the paths
        for (level, count) in &nodes_per_level {
            tree_size += *level * *count;
        }

        // Space taken up by the counts
        let highest_count = nodes_per_level.iter().map(|(_, x)| *x).max().unwrap();
        tree_size += (highest_count.ilog2() + 1) * depth;
        tree_size = (tree_size as f64 / 8f64).ceil() as u32;
        tree_size += 3;

        // println!("Calculation finished for bit size {}", chunk_size);
        Ok(Prediction {
            chunk_size,
            tree_size,
            file_size,
            total_size: tree_size as u64 + file_size + FILE_HEADER.len() as u64,
        })
    } else {
        Err(format!("Chunk size {} does not evenly divide the number of bits in the input", chunk_size))
    }
}