
pub mod db_helper {
    // ├── src
    // │   |--- main.rs
    // │   |--- A.rs
    // |   |--- B.rs
    // since A is imported in main, use crate::A in B
    use crate::run_py_struct::run_py_struct::RunPy;
    use chrono::prelude::*;
    use std::fs;
    use std::io::{self, BufRead};
    use std::collections::HashMap;
    use std::fs::File;
    use std::path::Path;
    use std::io::Write;


    pub fn read_env_file(filename: &str) -> io::Result<HashMap<String, String>> {
        let file = File::open(filename)?;
        let reader = io::BufReader::new(file);
    
        let mut result = HashMap::new();
    
        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() == 2 {
                let key = parts[0].trim().to_string();
                let value = parts[1].trim().to_string();
                result.insert(key, value);
            }
        }
        Ok(result)
    }

    pub fn seed_database(db_p: String) {
        // Extract the directory part from the file path and store it in a variable
        let directory_path: &str = get_directory_from_path(db_p.as_str());
    
        if !directory_path.is_empty() {
            // Check if it exists and create if necessary
            if !fs::metadata(directory_path).is_ok() {
                fs::create_dir_all(directory_path).expect("Failed to create directory");
            }
        } else {
            println!("No directory part found.");
        }
    
        // Create folder if doesn't exist
        match create_folder_if_not_exists(directory_path) {
            Ok(_) => println!("Data Folder created or already exists."),
            Err(e) => println!("Error creating data folder: {}", e),
        }
    
        // Create the JSON file if doesn't exist
        if let Err(e) = create_json_if_not_exists(db_p.as_str()) {
            println!("Error creating JSON file: {}", e);
        } else {
            println!("JSON file created or already exists.");
        }
    }

    fn get_directory_from_path(file_path: &str) -> &str {
        let path = Path::new(file_path);
        // Get the parent path, convert it to `&str`, and provide a default if `None`
        path.parent()
            .and_then(|p| p.to_str()) // Convert to `<&str>`
            .unwrap_or("./data") // Default to root if the parent or conversion fails
    }

    fn create_folder_if_not_exists(folder_path: &str) -> io::Result<()> {
        if !fs::metadata(folder_path).is_ok() {
            // This will create all necessary intermediate directories
            fs::create_dir_all(folder_path)?;
        }
        Ok(())
    }
    
    fn create_json_if_not_exists(file_path: &str) -> io::Result<()> {
        // for startup use
        let path = Path::new(file_path);
        if !path.exists() {
            let db_seed = build_seeds();
    
            // Open a file in write mode
            let mut file: File = std::fs::File::create(file_path)?;
    
            // Write the JSON data to the file
            write!(file, "{}", serde_json::to_string_pretty(&db_seed)?)?;
        }
        Ok(())
    }

    pub fn overwrite_json(db_addr: String) -> io::Result<()> {
        // for initialize flag use
        let db_seed = build_seeds();
    
        // Open a file in write mode
        let mut file = std::fs::File::create(db_addr)?;
    
        // Write the JSON data to the file
        write!(file, "{}", serde_json::to_string_pretty(&db_seed)?)?;
        Ok(())
    }

    fn build_seeds() -> Vec<RunPy>{
        let mut seeds: Vec<RunPy> = vec![];
        seeds.push(RunPy {
            id: 1,
            py_script: "default_script.py".to_string(),
            description: "init description".to_string(),
            created_at: Utc::now(),
        });

        // Iterate over the files in the directory
        for entry in fs::read_dir(".").expect("loop directory issuse") {
            let entry = entry.expect("can't read inside the directory");
            let path = entry.path();

            // Check if the entry is a file with a `.py` extension
            if let Some(extension) = path.extension() {
                if extension == "py" {
                    seeds.push(RunPy {
                        id: 1,
                        py_script: path.display().to_string(),
                        description: "found it".to_string(),
                        created_at: Utc::now(),
                    });
                }
            }
        }
        seeds
    }
}

