
#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

// launch dev mode with:
// cargo tauri dev

use std::fs::*;
use std::collections::HashMap;
use std::io::Read;
use std::io::Write;
use chrono::{DateTime, Local}; // TimeZone, NaiveDateTime
use chrono::TimeZone;
use std::path::Path;
// use std::cmp::Ordering;
use std::process::Command;
use std::error::Error;
use std::sync::Mutex;
// use std::thread;
// use std::time::Duration;
use lazy_static::lazy_static; // 1.4.0



mod segment;
use segment::Segment;

mod tauri_commands;
use tauri_commands::*;

mod server_http;
use server_http::start_httpserver;

fn save_json(filename:&String, data_to_save:&Vec<HashMap<String, String>>)
{
	#[cfg(debug_assertions)] { println!("Saving {}...", filename); }
	
	let mut file = match File::create(filename) {
		Ok(file) => file,
		Err(e) => {
			println!("Error creating file: {:?}", e);
			return;
		}
	};
	let json_data = serde_json::to_string_pretty(&data_to_save).unwrap();
	let out = file.write_all(json_data.as_bytes());

	#[cfg(debug_assertions)] {
		match out{
			Ok(_)         => println!("File saved successfully"),
			Err(e) => println!("Error saving file: {:?}", e),
		}
	}
}


// parse json file to list of hashmaps
pub fn load_json(file_name: &str) -> Result<Vec<HashMap<String, String>>, String> {

	// Check that path exists
	if !Path::new(file_name).exists() {
		panic!("'{}' does not exist", file_name);
	}

	#[cfg(debug_assertions)]
	{
		println!("Reading... {}", file_name);
	}

	let mut file = File::open(file_name).unwrap();
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
pub fn list_files_in_folder(folder_name: &str) -> Result<Vec<String>, String> {

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





// Create a list of segments with the info of the jsons in the data folder
pub fn list_of_segments(path_dir:&String) -> Vec<Segment> {

	let mut to_return : Vec<Segment> = Vec::new();

	let mut files = list_files_in_folder(&path_dir).unwrap();
	// for file in &files {
	// println!("file: {}", file);
	// }

	// Filter only files that end with .json
	files = files.into_iter().filter(|x| x.ends_with(".json")).collect();

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
			start: Local.datetime_from_str(&i.get("start").unwrap(), "%Y-%m-%d %H:%M:%S").unwrap(),
			end  : Local.datetime_from_str(&i.get("end"  ).unwrap(), "%Y-%m-%d %H:%M:%S").unwrap(),
		});
	}

	// WIP, sort this vector once ord is implemented in Segment

	return to_return;
}


pub fn execute_script_python(script_to_execute : &str) -> Result<(), Box<dyn Error>>
{
	println!("{}", script_to_execute);

	// We check the script exists
	if Path::new(script_to_execute).exists()
	{
		let output_command = Command::new("pythonw")
			.arg(script_to_execute)
			.spawn();
		if output_command.is_err() { println!("Python script gave an error: {:?}", output_command.err().unwrap()); }
		else {                       println!("Python script executed successfully"); }
	}
	else
	{
		println!("File does not exist: {}", script_to_execute);
	}

	Ok(())
}




// static PATH_ROOT_FOLDER : &str = "C:/data_pomodoros";
const  PATH_ROOT_FOLDER : &str       = "C:/Registries/Pomodoro";
// static mut timer_total_s : i32 = 2 * 60;
// static timer_total_s      : Mutex<i32>;
// static timer_total_s      : Mutex<i32> = Mutex::new(2 * 60);
// const timer_total_s      : Mutex<i32> = Mutex::new(2 * 60);

lazy_static! {
	// static timer_total_s : Mutex<i32> = Mutex::new(2 * 60);
	static ref TIMER_TOTAL_S : Mutex<i32> = Mutex::new(2 * 60);

	// datetime
	static ref START_TIME    : Mutex<DateTime<Local>> = Mutex::new(Local::now());
}

fn main() {

	#[cfg(debug_assertions)] { println!("Starting backend..."); }

	// Create path with intermediate folders if it doesn't exist
	if !Path::new(&PATH_ROOT_FOLDER).exists() {
		#[cfg(debug_assertions)] { println!("Created missing path..."); }
		create_dir_all(&PATH_ROOT_FOLDER).unwrap();
	}

	// timer_total_s = Mutex::new(2 * 60);

	std::thread::spawn(start_httpserver);


	// // List of segments
	// let mut _out = list_of_segments(&PATH_ROOT_FOLDER.to_string());
	// _out.sort();
	// // for i in _out.iter() {println!("{:?}", i);}

	// // We get the unique names of the segments
	// let mut unique_names : Vec<String> = Vec::new();
	// for i in _out.iter().rev() {
	// 	if !unique_names.contains(&i.name) {
	// 		unique_names.push(i.name.clone());
	// 	}
	// }
	// print !("{:?}", unique_names);

	// println!("Writting...");
	// annotate_pomodoro("First".to_string(), None);

	tauri::Builder::default()
		.invoke_handler(tauri::generate_handler![
			command_retrieve_last_pomodoros,
			annotate_pomodoro,
			conf_get_time_pomodoro_in_min,
			get_last_date_of_segment,
			pomodoro_start,
			pomodoro_end,
		])
		.run(tauri::generate_context!())
		.expect("error while running tauri application");

}
