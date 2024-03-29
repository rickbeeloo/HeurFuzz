
use std::collections::HashMap;
use ndarray::{Array2, Array, Ix2};
use std::io::BufReader;
use std::fs::File;
use std::io::BufRead;
use clap::{Arg, App};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use fuzzywuzzy::fuzz;
use std::io::Write;

const MAGNITUDE: i32 = 2_i32.pow(30);

#[derive(Debug, PartialEq, Eq)]
struct Entry {
    ref_index: u32,
    coverage: i32,
    length: i32,
}

impl Ord for Entry {
    fn cmp(&self, other: &Self) -> Ordering {
        match other.coverage.cmp(&self.coverage) {
            Ordering::Equal => self.length.cmp(&other.length), // Compare length if coverage is equal
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
    if heap.len() < 20 {
        heap.push(entry);
    } else if let Some(smallest) = heap.peek() {
        if entry < *smallest {
            let _ = heap.pop();
            heap.push(entry);
        }
    }
}

fn transform(ref_index: u32, coverage: u32, length: i32) -> Entry {
    Entry{ref_index, coverage: (coverage as i32) - MAGNITUDE, length: length - MAGNITUDE}
}

fn reverse_transform(entry: &Entry) -> (u32, i32, i32) {
    let coverage: i32 = MAGNITUDE + entry.coverage;
    let length: i32= MAGNITUDE + entry.length;
    (entry.ref_index, coverage, length)
}

fn read_lines_to_uint8_vector(file_path: &str) -> Result<Vec<Vec<u8>>, std::io::Error> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut output = Vec::new();
    for line_result in reader.lines() {
        let line = line_result?.to_lowercase();
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


fn create_frequency_map(bigrams: &Vec<(u8, u8)>) -> HashMap<(u8, u8), usize> {
    let mut frequency_map = HashMap::with_capacity(bigrams.len());
    for &bigram in bigrams {
        *frequency_map.entry(bigram).or_insert(0) += 1;
    }
    frequency_map
}

fn heuristic_filter(index: &HashMap<(u8, u8), HashMap<usize, u32>>, refs: &[Vec<u8>], query_lens: &Vec<u32>, cov_vector: &mut Vec<u32>, heaps: &mut Vec<BinaryHeap<Entry>> ) {
    refs.iter().enumerate().for_each(|(j, r)| {
        
        // Reset coverage vec
        cov_vector.iter_mut().for_each(|m| *m = 0);

        // Show the progress
        if j % 10_000 == 0 {
            let percent = j / refs.len(); 
            println!("Proccessed: {}/{}", j, refs.len());
        }
        
        // Create bigram reference frequency map and compare it with each query map
        if r.len() > 1 {
            let r_bigram_map: HashMap<(u8, u8), usize> = create_frequency_map(&generate_bigrams(r));
            for (bigram, ref_count) in r_bigram_map.iter() {
                if let Some(entry) = index.get(&bigram) {
                    for (query_id, query_count) in entry {
                        cov_vector[*query_id] += std::cmp::min(*query_count, *ref_count as u32); 
                    }
                }
            }
        } else {
            println!("Discarded: {:?}", r);
        }
       
        // For each query we now have the coverage with the current ref
        // we can push these to the heaps
        cov_vector.iter().enumerate().for_each(|(query_index, coverage)|{
            let q_len = query_lens[query_index] as i32;
            let length_difference = (r.len() as i32 - q_len).abs();
            update_heap(&mut heaps[query_index], transform(j as u32, *coverage, length_difference));
        }) 
    });
}


fn replace_non_utf8_with_space(bytes: &mut Vec<u8>) {
    for byte in bytes.iter_mut() {
        if !byte.is_ascii() {
            // Replace non-UTF-8 byte with space
            *byte = 32u8; // Space character
            println!("[WARNING] Replaced a non UTF-8 byte");
        }
    }
}


fn bytes_to_utf8_string(bytes: &[u8]) -> String {
    let mut modified_bytes = bytes.to_vec();
    replace_non_utf8_with_space(&mut modified_bytes);
    String::from_utf8_lossy(&modified_bytes).into_owned()
}

fn fuzz_pass(heaps: &mut Vec<BinaryHeap<Entry>>, queries: &[Vec<u8>], refs: &[Vec<u8>], cut_off: u8, score_scale: f32, output_path: &str) {
    let mut output_file = File::create(output_path)
        .expect("Failed to create output file");

    // Write the header
    writeln!(output_file, "query\treference")
        .expect("Failed to write header to output file");

    // Keep track of how many items could not be mapped
    let mut not_mapped = 0;

    for (index, (bytes, heap)) in queries.iter().zip(heaps.iter_mut()).enumerate() {
        let query_string = bytes_to_utf8_string(bytes);
        // println!("Q: {}", query_string);
        // println!("Heap");

        // for item in heap.iter() {
        //     let ref_bytes = &refs[item.ref_index as usize];
        //     let ref_string = String::from_utf8_lossy(ref_bytes);
        //     let (_, c, l) = reverse_transform(&item);
        //     println!("{}: c:{} l:{}", ref_string, c, l );
        // }

        let max_match = find_max_match(heap, refs, &query_string, cut_off, score_scale);

        if max_match.is_empty() {
            not_mapped +=1;
        }

        // Write query and best match to output file
        writeln!(output_file, "{}\t{}", query_string, max_match)
            .expect("Failed to write query and best match to output file");

        // Print progress
        if index % 100 == 0 {
            println!("Processed query {} of {}", index + 1, queries.len());
        }
        
    }

    println!("Done fuzzing: {} / {} mapped!", queries.len() - not_mapped, queries.len() );
}

fn find_max_match(heap: &mut BinaryHeap<Entry>, refs: &[Vec<u8>], query_string: &str, cut_off: u8, score_scale: f32) -> String {
    let mut max_score = 0;
    let mut max_match = String::new();
    let mut last_size_difference: i32 = 1_000_000;

    while let Some(item) = heap.pop() {
        let (ref_index, coverage, l) = reverse_transform(&item);
        let ref_bytes = &refs[ref_index as usize];
        let ref_string = String::from_utf8_lossy(ref_bytes);

        let fuzz_r = fuzz::partial_ratio(&ref_string, query_string);
        
        
        let size_difference = (ref_bytes.len() as i32 - query_string.len() as i32).abs();
        
        let combined_score = (fuzz_r as f32 * score_scale) as i32 - size_difference;

        //println!("q:{}\tr:{}\ts:{} l:{}, score: {}\n", query_string, ref_string, fuzz_r, size_difference, combined_score);

        if fuzz_r >= cut_off && (combined_score > max_score || (max_score == combined_score && size_difference < last_size_difference)) {
            max_match = ref_string.into_owned(); 
            max_score = combined_score;
            last_size_difference = size_difference;
        }
    }

    max_match
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
        .arg(Arg::with_name("cutoff")
            .help("Fuzzing score cut-off, e.g. 90 = 90% match between reference and query")
            .required(true)
            .index(3))
        .arg(Arg::with_name("output")
            .help("Output path")
            .required(true)
            .index(4))
        .arg(Arg::with_name("scale")
            .help("Scale to score the fuzzing score with compared to length difference, e.g. 2 = 2*score - length (DEFAULT: 2)")
            .index(5)
            .default_value("2")) // Set default value here
        .get_matches();

    
    let query_path = matches.value_of("query").unwrap();
    let ref_path = matches.value_of("reference").unwrap();
    let output_path = matches.value_of("output").unwrap();
    let cut_off: u8 = matches.value_of("cutoff").unwrap().parse().expect("Cut off not a number");
    let score_scale: f32 = matches.value_of("scale").unwrap().parse().expect("Cut off not a number");

    println!("Reading data...");
    let query_vector = read_lines_to_uint8_vector(query_path).expect("Error reading query");
    let ref_vector = read_lines_to_uint8_vector(ref_path).expect("Error reading reference");

    println!("Index queries in hashmap...");
    let index = index_queries(&query_vector);

    println!("Querying...");
    let mut cov_vector = vec![0;query_vector.len()];
    let mut len_vector: Vec<u32> = query_vector.iter().map(|sublist| sublist.len() as u32).collect();
    let mut heaps: Vec<BinaryHeap<Entry>> = Vec::new();
    for _ in 0..query_vector.len() {
        heaps.push(BinaryHeap::new());
    }

    heuristic_filter(&index, &ref_vector, &len_vector, &mut cov_vector, &mut heaps);

    // for heap in heaps {
    //     println!("{:?}", heap);
    // }

    fuzz_pass(&mut heaps, &query_vector, &ref_vector, cut_off, score_scale, &output_path);
    
    println!("Done!");
}

// fn main() {
//     let queries = vec![vec![1, 1, 2], vec![1, 2, 3]];
//     let refs = vec![vec![1, 1, 2, 3], vec![1, 1, 3], vec![1, 2, 3]];
//     let m = Array2::<u32>::zeros((refs.len(), queries.len()));
    
//     let index = index_queries(&queries);
//     let mat = query_index(&index, &refs, m);

//     println!("{:?}", mat);
// }


// a yellow banana from the store
// vanilla
// peanutbutter