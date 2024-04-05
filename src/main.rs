use std::{env, fs};
use std::error::Error;
use std::path::{Path, PathBuf};

use configparser::ini::Ini;
use csv::ReaderBuilder;
use walkdir::WalkDir;

fn main() {
    match env::current_exe() {
        Ok(exe_path) => {
            // load program configuration
            let mut config = Ini::new();
            let mut path = exe_path.clone();

            // save current working directory
            path.pop();
            let cwd = path.clone();

            // load config file at cwd
            path.push("backup.ini");
            let config_path = path;
            let map = config.load(config_path);
            println!("configuration: {:?}", map);

            // paths that will be saved to current working directory
            let paths_config = config.get("DEFAULT", "paths");
            let paths = csv_to_vec(paths_config.unwrap().as_str()).unwrap();

            // file extensions that will be ignored
            let excludes = csv_to_vec(config.get("DEFAULT", "excludes")
                .unwrap()
                .as_str())
                .unwrap();

            // will directory be cleaned before copy
            let clean = config.get("DEFAULT", "clean")
                .unwrap()
                .as_str()
                .eq_ignore_ascii_case("true");

            // clean all files but this application if enabled
            if clean {
                clean_files(cwd.clone())
            }

            // copy files to current working directory
            copy_files(paths, excludes, cwd)
        },
        Err(e) => println!("failed to get current working directory: {}", e)
    }
}

fn csv_to_vec(csv_data: &str) -> Result<Vec<String>, Box<dyn Error>>{
    // prepare data structures
    let mut reader = ReaderBuilder::new().from_reader(csv_data.as_bytes());
    let mut records = Vec::new();

    // parse single value or csv, and add to vector
    if !csv_data.contains(",") && csv_data.len() > 0 {
        records.push(String::from(csv_data));
    } else {
        if let Ok(result) = reader.headers() {
            let record = result;
            for rec in record {
                records.push(rec.to_string());
            }
        }
    }

    Ok(records)
}

fn clean_files(cwd: PathBuf) {
    println!("cleaning files");
    for entry in WalkDir::new(cwd) {
        let entry = entry.unwrap();
        let path = entry.path();
        match path.file_name() {
            None => {
                continue;
            }
            Some(file_path) => {
              if file_path.eq_ignore_ascii_case("backup.ini")
                 || file_path.eq_ignore_ascii_case("backup.exe") {
                  continue;
              } else {
                  if path.is_dir() {
                      println!("removing: {}", path.display());
                      fs::remove_dir_all(path).expect("failed to clean directory: {}");
                  } else if path.exists()  {
                      println!("removing: {}", path.display());
                      fs::remove_file(path).expect("failed to clean file");
                  }
              }
            }
        }
    }
}

fn copy_files(paths: Vec<String>, excludes: Vec<String>, cwd: PathBuf) {
    println!("start copy files: {}", paths.len());

    // For each directory marked for copy
    for path_str in paths {
        println!("path string: {}", path_str);
        let path_buf = Path::new(&path_str);

        // for each file in that directory
        for entry in WalkDir::new(path_buf) {
            // get path of file
            let entry = entry.unwrap();
            let path = entry.path();

            // use path buffer to copy to current working directory
            let mut destination = cwd.clone();
            let path_buffer = PathBuf::from(path);

            // root directory buffer is absolute
            let mut mut_path_buffer = PathBuf::from(&path_str);
            mut_path_buffer.pop();

            // strip
            let stripped = path_buffer
                .strip_prefix(mut_path_buffer)
                .expect("TODO: panic message");
            destination.push(stripped);

            // create directories
            if path.is_dir() {
                println!{"create directory: {}", destination.display()}
                fs::create_dir_all(destination).unwrap();

            // copy files
            } else {
                // check excluded extensions
                match path.extension() {
                    None => {
                        println!("copy {} to {}", path.display(), destination.as_path().display());
                        fs::copy(&path, &destination.as_path()).unwrap();
                    },
                    Some(extension) => {
                        let mut excluded = false;
                        for exclude in &excludes {
                            let exclude_extension = exclude.replace(".", "");
                            println!("exclude extensions: {}", exclude_extension.as_str());
                            println!("extension: {}", extension.to_str().unwrap());
                            if exclude_extension.as_str() == extension {
                                excluded = true;
                            }
                        }

                        if!excluded {
                            // copy files
                            println!("copy {} to {}", path.display(), destination.as_path().display());
                            fs::copy(&path, &destination.as_path()).unwrap();
                        }
                    },
                }
            }
        }
    }
}
