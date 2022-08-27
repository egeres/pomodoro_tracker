
#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

// launch dev mode with:
// cargo tauri dev

use std::fs::*;
use std::collections::HashMap;
use std::io::Read;
use chrono::{DateTime, Utc, FixedOffset}; // TimeZone, NaiveDateTime
use chrono::TimeZone;
use std::path::Path;
use std::cmp::Ordering;

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



#[derive(Debug)]
struct Segment {
  name : String,
  start: DateTime<Utc>,
  end  : DateTime<Utc>,
}

impl Ord for Segment { // https://doc.rust-lang.org/std/cmp/trait.Ord.html
    fn cmp(&self, other: &Self) -> Ordering {
        self.start.cmp(&other.start)
    }
}

impl PartialOrd for Segment {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Segment {
    fn eq(&self, other: &Self) -> bool {
        (self.start == other.start) && (self.name == other.name) && (self.end == other.end)
    }
}

impl Eq for Segment {}

// impl Eq for Segment { 
// 	fn eq(&self, other: &Self) -> bool {
//         (self.start == other.start) && (self.name == other.name) && (self.end == other.end)
// 	}
// }

// Create a list of segments with the info of the jsons in the data folder
fn list_of_segments(path_dir:String) -> Vec<Segment>
{
	let mut to_return : Vec<Segment> = Vec::new();

	let files = list_files_in_folder(&path_dir).unwrap();
	// for file in &files {
	// println!("{}", file);
	// }

	let data_of_files_merged = files.iter().map(|file_name| {
		load_json(&(path_dir.to_owned()+"/"+file_name)).unwrap()
	}).flatten().collect::<Vec<HashMap<String, String>>>();
	// for item in data_of_files_merged {
	// 	println!("{:?}", item);
	// }

	for i in data_of_files_merged.iter()
	{
		to_return.push(Segment {
			name : i.get("name").unwrap_or(&"unknown".to_string()).to_string(),
			start: Utc.datetime_from_str(&i.get("start").unwrap(), "%Y-%m-%d %H:%M:%S").unwrap(),
			end  : Utc.datetime_from_str(&i.get("end"  ).unwrap(), "%Y-%m-%d %H:%M:%S").unwrap(),
		});
	}

	// WIP, sort this vector once ord is implemented in Segment

	return to_return;
}

#[tauri::command]
fn command_retrieve_last_pomodoros() -> Vec<String> {

	println!("command_retrieve_last_pomodoros");

	vec![
		"First" .to_string(),
		"Second".to_string(),
		"Third" .to_string(),
	]
}

fn main() {

	println!("Starting...");

	let path_root_folder = "data".to_string();

	if !Path::new(&path_root_folder).exists() {
		// Create path with intermediate folders if it doesn't exist
		create_dir_all(&path_root_folder).unwrap();
	}


	let mut _out = list_of_segments(path_root_folder);

	_out.sort();

	for i in _out.iter() {println!("{:?}", i);}

	let _o = 0;

	tauri::Builder::default()
		.invoke_handler(tauri::generate_handler![command_retrieve_last_pomodoros])
		.run(tauri::generate_context!())
		.expect("error while running tauri application");

}
