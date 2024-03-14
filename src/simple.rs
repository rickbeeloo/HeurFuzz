use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;


fn read_lines(file_path: &str) -> io::Result<Vec<String>> {
    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);
    let mut lines = Vec::new();

    for line in reader.lines() {
        let line = line?;
        lines.push(line.trim().to_string());
    }

    Ok(lines)
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
    
    let query_lines = read_lines(query_path);
    

}

