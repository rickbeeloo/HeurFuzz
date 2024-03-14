use std::fs::File;
use std::io::{BufReader, BufRead};
use std::collections::HashMap;
use clap::{Arg, App};
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;

fn read_lines_to_uint8_vector(file_path: &str) -> Result<Vec<Vec<u8>>, std::io::Error> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut output = Vec::new();
    for line_result in reader.lines() {
        let line = line_result?;
        let uint8_vec: Vec<u8> = line.bytes().collect();
        output.push(uint8_vec);
    }

    Ok(output)
}

fn as_bigram_map(vec: &Vec<u8>) -> HashMap<Vec<u8>, usize> {
    let mut bigram_counts: HashMap<Vec<u8>, usize> = HashMap::new();
    for bigram in vec.windows(2) {
        *bigram_counts.entry(bigram.to_vec()).or_insert(0) += 1;
    }
    bigram_counts
}

fn compare_against_map(map: &HashMap<Vec<u8>, usize>, vec:&Vec<u8>) -> u32 {
    let mut shared_count : u32 = 0;
    for bigram in vec.windows(2) {
        if let Some(&count) = map.get(&bigram.to_vec()) {
            shared_count += count as u32;
        }
    }
    shared_count
}

// fn build_coverage_matrix(query_vector: &Vec<Vec<u8>>, ref_vector: &Vec<Vec<u8>>) {
//     let mut cov_vector: Vec<u32> = vec![0; ref_vector.len()];
//     for (qpos, q) in query_vector.iter().enumerate() {
//         println!("Working on {}", qpos);
//         let query_bigramp_map = as_bigram_map(&q);
//         for (rpos, r) in ref_vector.iter().enumerate() {
//             cov_vector[rpos] = compare_against_map(&query_bigramp_map, &r);
//         }
//         let maxValue = cov_vector.iter().max();
//         println!("Max value: {:?}", maxValue);
//     }
// }

fn build_coverage_matrix(query_vector: &Vec<Vec<u8>>, ref_vector: &Vec<Vec<u8>>) {

    ThreadPoolBuilder::new()
        .num_threads(70)
        .build_global()
        .unwrap();

    query_vector.par_iter().enumerate().for_each(|(qpos, q)| {
        println!("Working on {}", qpos);
        let mut cov_vector: Vec<u32> = vec![0; ref_vector.len()];
        let query_bigramp_map = as_bigram_map(&q);
        for (rpos, r) in ref_vector.iter().enumerate() {
            cov_vector[rpos] = compare_against_map(&query_bigramp_map, &r);
        }
        let max_value = cov_vector.iter().max();
        println!("Max value: {:?}", max_value);
    });
}

fn build_len_matrix(query_vector: &Vec<Vec<u8>>, ref_vector: &Vec<Vec<u8>>) -> Vec<Vec<u32>> {
    let mut mat: Vec<Vec<u32>> = vec![vec![0; ref_vector.len()]; query_vector.len()];
    for (qpos, q) in query_vector.iter().enumerate() {
        for (rpos, r) in ref_vector.iter().enumerate() {
            mat[qpos][rpos] = (q.len() as i32 - r.len() as i32).abs() as u32;
        }
    }
    mat
}



fn main() {
    let matches = App::new("Matrix Builder")
        .arg(Arg::with_name("query")
            .help("Path to the query file")
            .required(true)
            .index(1))
        .arg(Arg::with_name("reference")
            .help("Path to the reference file")
            .required(true)
            .index(2))
        .get_matches();

    
    let query_path = matches.value_of("query").unwrap();
    let ref_path = matches.value_of("reference").unwrap();

    println!("Reading data...");
    let query_vector = read_lines_to_uint8_vector(query_path).expect("Error reading query");
    let ref_vector = read_lines_to_uint8_vector(ref_path).expect("Error reading reference");

    println!("Building coverage matrix...");
    build_coverage_matrix(&query_vector, &ref_vector);

}