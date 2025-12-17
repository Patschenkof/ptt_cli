use std::{fs, env};
use std::path::{PathBuf};
use serde::{Serialize};
use serde::de::DeserializeOwned;

/*
pub fn save_data<T: Serialize>(data: &Vec<T>, file_name: &str) { // <T: Serialize> tells rust it shoudl be any type (T) as long as it has the trait serialize
    // Need to overwrite file if already existing and to create new file if it does not
    let file_path = get_data_file_path(file_name);

    if let Ok(json) = serde_json::to_string_pretty(data) {
        if let Err(e)= fs::write(file_path, json){
        println!("Failed to save file. Error: {}", e);
        } else {
        println!("Data saved successfully!");
        }
    } else {
        println!("Failed to serialize data!");
    }          

    /*

    if let Ok(json) = serde_json::to_string_pretty(data) {
        let _ = fs::write(file_path, json); // fs::write returns a value. lev _ ignores it
    }
    */
}

pub fn load_data<T: DeserializeOwned>(file_name: &str) -> Vec<T> {
    let file_path = get_data_file_path(file_name);

    if file_path.exists() {
        let contents = fs::read_to_string(file_path).unwrap_or_else(|_| String::new());

        serde_json::from_str(&contents).unwrap_or_else(|_| Vec::new())
    } else {
        Vec::new()
    }
}

pub fn get_data_file_path(file_name: &str) -> PathBuf {
    let mut path = env::current_dir().unwrap(); // get current working dir
    path.push(file_name); // append filename
    path
}

*/
