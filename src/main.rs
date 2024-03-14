
use std::collections::HashMap;
use ndarray::{Array2, Array, Ix2};
use std::io::BufReader;
use std::fs::File;
use std::io::BufRead;
use clap::{Arg, App};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use fuzzywuzzy::fuzz;

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
    if heap.len() < 2 {
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

fn reverse_transform(entry: Entry) -> (u32, i32, i32) {
    let coverage: i32 = MAGNITUDE + entry.coverage;
    let length: i32= MAGNITUDE + entry.length;
    (entry.ref_index, coverage, length)
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
    println!("Index: {:?}", index);
    index
}


fn heuristic_filter(index: &HashMap<(u8, u8), HashMap<usize, u32>>, refs: &[Vec<u8>], query_lens: &Vec<u32>, cov_vector: &mut Vec<u32>, heaps: &mut Vec<BinaryHeap<Entry>> ) {
    refs.iter().enumerate().for_each(|(j, r)| {
        
        // Reset coverage vec
        cov_vector.iter_mut().for_each(|m| *m = 0);

        // Show the progress
        if j % 100_000 == 0 {
            println!("Proccessed: {}/{}", j, refs.len());
        }
        
        // Not ideal, but lets keep track of the speicif counts
        let mut filter = HashMap::new();

        // Fill coverage matrix
        if r.len() > 2 {
            for bigram in generate_bigrams(r) {
                if let Some(entry) = index.get(&bigram) {
                    for (query_id, count) in entry {
                        let query_filter = filter.entry(*query_id).or_insert_with(HashMap::new);
                        // Increment cov_vector and update filter
                        if let Some(filter_count) = query_filter.get_mut(&bigram) {
                            if *filter_count < *count {
                                cov_vector[*query_id] += 1;
                                *filter_count += 1;
                            }
                        } else {
                            *query_filter.entry(bigram.clone()).or_insert(0) += 1;
                            cov_vector[*query_id] += 1;
                        }
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


fn fuzz_pass(heaps: &mut Vec<BinaryHeap<Entry>>, queries: &[Vec<u8>], refs: &[Vec<u8>], cut_off: u8) {
    for (bytes, heap) in queries.iter().zip(heaps.iter_mut()) {
        let query_string = String::from_utf8_lossy(bytes);

        // println!("Query: {}", query_string);
        // println!("Heap");
        // while let Some(item) = heap.pop() {
        //     let (ref_id, c, l) = reverse_transform(item);
        //     let ref_bytes = &refs[ref_id as usize];
        //     let ref_string = String::from_utf8_lossy(ref_bytes);
        //     println!("{}: c:{} l:{}", ref_string, c, l);
        // }
        let (max_match, max_score, max_len) = find_max_match(heap, refs, &query_string, cut_off);
        println!("Best match: {}", max_match);
        println!(" ---- END ----")
    }
}

fn find_max_match(heap: &mut BinaryHeap<Entry>, refs: &[Vec<u8>], query_string: &str, cut_off: u8) -> (String, u8, usize) {
    let mut max_score = 0;
    let mut max_match = String::new();
    let mut max_len = 0;

    while let Some(item) = heap.pop() {
        let (ref_index, coverage, l) = reverse_transform(item);
        let ref_bytes = &refs[ref_index as usize];
        let ref_string = String::from_utf8_lossy(ref_bytes);

        let fuzz_r = fuzz::partial_ratio(&ref_string, query_string);
        //println!("{} : {}: {} ", query_string, ref_string, fuzz_r);

        if fuzz_r >= cut_off && (fuzz_r > max_score || (max_score == fuzz_r && ref_bytes.len() > max_len)) {
            max_match = ref_string.into_owned(); 
            max_score = fuzz_r;
            max_len = ref_bytes.len();
        }
    }

    (max_match, max_score, max_len)
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
            .help("Fuzzing score cut-off")
            .required(true)
            .index(3))
        .get_matches();

    
    let query_path = matches.value_of("query").unwrap();
    let ref_path = matches.value_of("reference").unwrap();
    let cut_off: u8 = matches.value_of("cutoff").unwrap().parse().expect("Cut off not a number");

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

    fuzz_pass(&mut heaps, &query_vector, &ref_vector, cut_off);
    
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