
use std::collections::HashMap;
use ndarray::{Array2, Array, Ix2};
use std::io::BufReader;
use std::fs::File;
use std::io::BufRead;
use clap::{Arg, App};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

const MAGNITUDE: i32 = 2_i32.pow(30);

#[derive(Debug, PartialEq, Eq)]
struct Entry {
    coverage: i32,
    length: i32,
}

impl Ord for Entry {
    fn cmp(&self, other: &Self) -> Ordering {
        match other.coverage.cmp(&self.coverage) {
            Ordering::Equal => other.length.cmp(&self.length), // Compare length if coverage is equal
            ord => ord, // Return coverage ordering if not equal
        }
    }
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn update_heap(heap: &mut BinaryHeap<Entry>, entry: Entry) {
    if heap.len() < 2 {
        heap.push(entry);
    } else if let Some(smallest) = heap.peek() {
        if entry < *smallest {
            let _ = heap.pop();
            heap.push(entry);
        }
    }
}

fn transform(coverage: i32, length: i32) -> Entry {
    Entry{ coverage: coverage - MAGNITUDE, length: length - MAGNITUDE}
}

fn reverse_transform(entry: Entry) -> (i32, i32) {
    let coverage: i32 = MAGNITUDE + entry.coverage;
    let length: i32= MAGNITUDE + entry.length;
    (coverage, length)
}

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

fn generate_bigrams(lst: &[u8]) -> Vec<(u8, u8)> {
    let mut result = Vec::new();
    for i in 0..lst.len() - 1 {
        result.push((lst[i], lst[i + 1]));
    }
    result
}

fn index_queries(queries: &[Vec<u8>]) -> HashMap<(u8, u8), HashMap<usize, u32>> {
    let mut index = HashMap::new();
    for (i, query) in queries.iter().enumerate() {
        for bigram in generate_bigrams(query) {
            let seq_entry = index.entry(bigram).or_insert_with(HashMap::new);
            *seq_entry.entry(i).or_insert(0) += 1;
        }
    }
    index
}


fn query(index: &HashMap<(u8, u8), HashMap<usize, u32>>, refs: &[Vec<u8>], cov_vector: &mut [Vec<u32>], len_vector: &mut [Vec<u32>] ) {
    refs.iter().enumerate().for_each(|(j, ref_)| {
        
        // Show the progress
        if j % 100 == 0 {
            println!("Proccessed: {}/{}", j, refs.len());
        }
        
        // Show 
        for bigram in generate_bigrams(ref_) {
            if let Some(entry) = index.get(&bigram) {

                // Update Q * R match
                for (query_id, count) in entry {
                    cov_vector[query_id] += count;
                }
            }
        }
    });
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

    println!("Index queries in hashmap...");
    let index = index_queries(&query_vector);

    println!("Querying...");
    let mut m = Array2::<u32>::zeros((ref_vector.len(), query_vector.len()));

    let mut cov_vector = vec![0;query_vector.len()];
    len mut len_vector = vec![0;query_vector.len()];
    get_coverage_matrix(&index, &ref_vector, &mut cov_vector, &mut len_vector);
    
    println!("Done!");
    println!("{:?}", m);

}

// fn main() {
//     let queries = vec![vec![1, 1, 2], vec![1, 2, 3]];
//     let refs = vec![vec![1, 1, 2, 3], vec![1, 1, 3], vec![1, 2, 3]];
//     let m = Array2::<u32>::zeros((refs.len(), queries.len()));
    
//     let index = index_queries(&queries);
//     let mat = query_index(&index, &refs, m);

//     println!("{:?}", mat);
// }