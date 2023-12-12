use std::env;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

fn get_chunk_size(size_str: &str) -> u64 {
    let size_str = size_str.trim().to_lowercase();
    let size = size_str.parse::<u64>().unwrap_or(0);
    println!("size: {}", size);
    match size_str.chars().last() {
        Some('b') => size,
        Some('k') => size * 1024,
        Some('m') => size * 1024 * 1024,
        Some('g') => size * 1024 * 1024 * 1024,
        _ => size,
    }
}

fn split_file(file_path: &str, chunk_size: u64) -> io::Result<()> {
    let file = File::open(file_path)?;
    let mut reader = io::BufReader::new(file);

    let mut buffer = vec![0; chunk_size as usize];
    let mut part_number = 1;

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        let part_file_path = format!("{}.part{:04}.split", file_path, part_number);
        let mut part_file = File::create(part_file_path)?;

        part_file.write_all(&buffer[..bytes_read])?;
        
        part_number += 1;
    }

    Ok(())
}

fn unsplit_files(file_path: &str) -> io::Result<()> {
    let path = Path::new(file_path);
    let dir_path = path
        .parent().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid file path provided"))?;
    let file_name = path
        .file_name().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid file name provided"))?
        .to_str().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Non UTF-8 file name provided"))?;


    let mut part_files: Vec<PathBuf> = fs::read_dir(dir_path)?
    .filter_map(|entry| {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() && path.to_str().unwrap_or("").starts_with(file_name) {
                    Some(path)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();
    
    part_files.sort();
    
    let mut output_file = File::create(file_path)?;

    for part_file in part_files {
        let mut part_file = File::open(part_file)?;
        io::copy(&mut part_file, &mut output_file)?;
    }
    
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("Usage: {} split <file> -s <chunk_size> OR ./splitter unsplit <file>", args[0]);
        return;
    }

    let command = &args[1];
    let file_path = &args[2];

    match command.as_str() {
        "split" => {
            if args.len() < 5 || args[3] != "-s" {
                println!("Usage: {} split <file> -s <chunk_size>", args[0]);
                return;
            }
            
            println!("Size: {}", args[4]);
            let chunk_size = get_chunk_size(&args[4]);
            if chunk_size == 0 {
                println!("Invalid chunk size");
                return;
            }

            match split_file(file_path, chunk_size) {
                Ok(_) => println!("File split successfully"),
                Err(err) => println!("Error splitting file: {}", err),
            }
        }
        "unsplit" => {
            match unsplit_files(file_path) {
                Ok(_) => println!("Files unsplit successfully"),
                Err(err) => println!("Error unsplitting files: {}", err),
            }
        }
        _ => println!("Invalid command"),
    }
}