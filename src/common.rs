use bincode;
use itertools::*;
use log::debug;
use memmap2::Mmap;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::process::Command;
use std::*;

pub type Index = (String, u32);
pub type IndexOffset = usize;


#[derive(Debug)]
pub enum CgVgError {
    LoadIndexOob(u32, u32)
}

pub fn load<'a>(idx: u32, file_path: &'a str, index_path: &'a str) -> Result<Index, CgVgError> {
    // Open the binary file
    let index_file = File::open(index_path).expect("cannot open file");
    let reader = std::io::BufReader::new(index_file);

    // Deserialize the array of u16 from the file
    let indexes: Vec<IndexOffset> = bincode::deserialize_from(reader).expect("error");

    if idx as usize >= indexes.len() {
        return Err(CgVgError::LoadIndexOob(idx, indexes.len() as u32));
    }

    let start = if idx == 0 {
        0
    } else {
        indexes[idx as usize - 1]
    } as usize;

    let end = indexes[idx as usize] as usize;

    // Memory-map the file
    let file = OpenOptions::new().read(true).open(file_path).unwrap();
    let mmap = unsafe { Mmap::map(&file).expect("map failed") };

    // Retrieve and deserialize tuples from the memory-mapped file
    debug!("get idx: {idx:?} at offsets {start:?} {end:?}");
    let data = &mmap[start..end];
    let tuple: Index = bincode::deserialize(data).expect("cannot derialize");

    Ok(tuple)
}

// First function that saves the entire Vec
pub fn save<'a>(tuples: Vec<Index>, file_path: &'a str, index_path: &'a str) {
    // Serialize tuples to a binary format
    let serialized_data: Vec<Vec<u8>> = tuples
        .iter()
        .map(|tuple| bincode::serialize(tuple).unwrap())
        .collect();

    // Determine the size of the file needed
    let _total_size: usize = serialized_data.iter().map(|data| data.len()).sum();

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(file_path)
        .expect(&format!("cannot open file {}", &file_path));

    let mut index = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(index_path)
        .expect("cannot open file");

    // Since max size of linux file is 255 chars, u16 should be enough for all cases to store
    // an u32 and the file path
    let mut offsets: Vec<IndexOffset> = vec![];
    let mut total_size: IndexOffset = 0;

    // Write the size of each tuple for indexing
    for data in &serialized_data {
        let size = data.len() as IndexOffset;
        total_size += size;
        offsets.push(total_size);
    }

    let encoded: Vec<u8> = bincode::serialize(&offsets).expect("cannot serialize");
    // debug!("try to write indexes {offsets:?} encoded as: {encoded:?}");

    index.write_all(&encoded).expect("cannot write index");
    index.sync_all().expect("sync index");

    // Write the actual data
    for data in &serialized_data {
        file.write_all(data).expect("cannot write data");
    }

    file.sync_all().expect("cannot sync");
}

/// Very adhoc function since I need to expand two paths
pub fn expand_paths<'a>(path1: &'a str, path2: &'a str) -> Result<(String, String), String> {
    // Yuck using sh to expand the path to handle path constructed with ~ or variabale ($HOME)
    // We might consider one of these options: https://blog.liw.fi/posts/2021/10/12/tilde-expansion-crates/ (a bit outdated though)
    let expand_tild = Command::new("sh")
        .arg("-c")
        .arg(format!("echo {} {}", path1, path2))
        .output();

    let (p1, p2) = String::from_utf8(expand_tild.expect("Command failed to run").stdout)
        .unwrap()
        .split_whitespace()
        .map(|s| String::from(s))
        .collect_tuple()
        .unwrap();

    Ok((p1, p2))
}
