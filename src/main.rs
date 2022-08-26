
#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]


use std::fs::*;
use std::collections::HashMap;
use std::io::Read;
use chrono::{DateTime, TimeZone, NaiveDateTime, Utc};
use std::path::Path;


// parse json file to list of hashmaps
fn load_json(file_name: &str) -> Result<Vec<HashMap<String, String>>, String> {

	// Check that path exists
	if !Path::new(file_name).exists() {
		panic!("'{}' does not exist", file_name);
	}

	let mut file     = File::open(file_name).unwrap();
	let mut contents = String::new();

	file.read_to_string(&mut contents).unwrap();
	let json_data: serde_json::Value = serde_json::from_str(&contents).unwrap();
	let json_data = json_data.as_array().unwrap();
	let mut json_data_vec: Vec<HashMap<String, String>> = Vec::new();
	for item in json_data {
		let item = item.as_object().unwrap();
		let mut item_map: HashMap<String, String> = HashMap::new();
		for (key, value) in item {
			item_map.insert(key.to_string(), value.as_str().unwrap().to_string());
		}
		json_data_vec.push(item_map);
	}
	Ok(json_data_vec)
}

// List files in a folder
fn list_files_in_folder(folder_name: &str) -> Result<Vec<String>, String> {

	// Check that path exists
	if !Path::new(folder_name).exists() {
		panic!("'{}' does not exist", folder_name);
	}

	let mut files: Vec<String> = Vec::new();
	for entry in read_dir(folder_name).unwrap() {
		let entry = entry.unwrap();
		let path = entry.path();
		let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
		files.push(file_name);
	}
	Ok(files)
}

struct segment {
  name : Option<String>,
  start: DateTime<Utc>,
  end  : DateTime<Utc>,
}

fn main() {

	println!("Starting server...");

	// let data = load_json("data/my_json_test.json").unwrap();

	let path_root_folder = "data";

	let files = list_files_in_folder(path_root_folder).unwrap();
	for file in &files {
	println!("{}", file);
	}

	let data_of_files_merged = files.iter().map(|file_name| {
	load_json(&(path_root_folder.to_owned()+"/"+file_name)).unwrap()
	}).flatten().collect::<Vec<HashMap<String, String>>>();
	for item in data_of_files_merged {
		println!("{:?}", item);
	}

	tauri::Builder::default()
	  .run(tauri::generate_context!())
	  .expect("error while running tauri application");

}
