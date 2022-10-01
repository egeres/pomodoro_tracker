
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
use std::cmp::Ordering;
use std::process::Command;

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
fn load_json(file_name: &str) -> Result<Vec<HashMap<String, String>>, String> {

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



#[derive(Debug, Clone)]
struct Segment {
  name : String,
  start: DateTime<Local>,
  end  : DateTime<Local>,
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
fn list_of_segments(path_dir:&String) -> Vec<Segment> {

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

#[tauri::command]
fn command_retrieve_last_pomodoros() -> Vec<String> {

	// List of segments
	let mut _out = list_of_segments(&PATH_ROOT_FOLDER.to_string());
	_out.sort();

	// We get the unique names of the segments
	let mut unique_names : Vec<String> = Vec::new();
	for i in _out.iter().rev() {
		if !unique_names.contains(&i.name) {
			unique_names.push(i.name.clone());
		}
	}

	unique_names
}

#[tauri::command]
fn annotate_pomodoro(pomodoro_name: String, duration_in_min:Option<i32>) -> Vec<String> {
	
	println!("dutati {}", duration_in_min.unwrap_or(999));

	let the_time : i32 = duration_in_min.unwrap_or(25);

	let this_segment = Segment
	{
		start: Local::now() - chrono::Duration::minutes(the_time as i64),
		end  : Local::now(),
		name : pomodoro_name,
	};

	#[cfg(debug_assertions)]
	{
		println!("Annotating pomodoro...");
		println!("\t Name : {}", this_segment.name );
		println!("\t Start: {}", this_segment.start);
		println!("\t End  : {}", this_segment.end  );
	}

	// We load the current information from scratch
	if !Path::new(&PATH_ROOT_FOLDER).exists() {
		create_dir_all(&PATH_ROOT_FOLDER).unwrap();
	}

	let mut all_segments = list_of_segments(&PATH_ROOT_FOLDER.to_string());
	all_segments.push(this_segment);
	all_segments.sort();

	let mut current_stack_of_segments : Vec<Segment> = vec![];
	if all_segments.len() > 0 {
		current_stack_of_segments.push(all_segments[0].clone());
	}

	for segment in all_segments.iter()
	{
		if all_segments.len() == 1
		{
			let filename         : String                       = format!("{}/{}.json", PATH_ROOT_FOLDER, current_stack_of_segments[0].start.format("%Y-%m-%d"));
			let mut data_to_save : Vec<HashMap<String, String>> = Vec::new();
			let s = segment;
			// for s in current_stack_of_segments.iter()
			// {
			let mut item_map : HashMap<String, String> = HashMap::new();
			item_map.insert("name" .to_string(), s.name.to_string());
			item_map.insert("start".to_string(), s.start.format("%Y-%m-%d %H:%M:%S").to_string());
			item_map.insert("end"  .to_string(), s.end  .format("%Y-%m-%d %H:%M:%S").to_string());
			data_to_save.push(item_map);
			// }
			save_json(&filename, &data_to_save);
		}

		// This left_right exists to gradually fill `current_stack_of_segments`, and emit a file when the stack is full
		// Each time `current_stack_of_segments` is full it represents a day
		let left    = format!("{}", segment.start.format("%Y-%m-%d"));
		let right_0 = current_stack_of_segments.last();
		let right_1 = right_0.unwrap().start;
		let right   = format!("{}", right_1.format("%Y-%m-%d"));

		if current_stack_of_segments.len() > 0 {
			if current_stack_of_segments.last().unwrap() == segment {
				continue;
			}
		}

		if left == right // The segment we have to append and the last one in the stack are from the same day
		{
			current_stack_of_segments.push((*segment).clone());
		}
		else
		{
			let filename         : String                       = format!("{}/{}.json", PATH_ROOT_FOLDER, current_stack_of_segments[0].start.format("%Y-%m-%d"));
			let mut data_to_save : Vec<HashMap<String, String>> = Vec::new();
			
			for s in current_stack_of_segments.iter()
			{
				let mut item_map : HashMap<String, String> = HashMap::new();
				item_map.insert("name" .to_string(), s.name.to_string());
				item_map.insert("start".to_string(), s.start.format("%Y-%m-%d %H:%M:%S").to_string());
				item_map.insert("end"  .to_string(), s.end  .format("%Y-%m-%d %H:%M:%S").to_string());
				data_to_save.push(item_map);
			}

			save_json(&filename, &data_to_save);

			current_stack_of_segments = Vec::new();
			current_stack_of_segments.push((*segment).clone());
		}
	}


	if current_stack_of_segments.len() > 0
	{
		let filename         : String                       = format!("{}/{}.json", PATH_ROOT_FOLDER, current_stack_of_segments[0].start.format("%Y-%m-%d"));
		let mut data_to_save : Vec<HashMap<String, String>> = Vec::new();
		for s in current_stack_of_segments.iter()
		{
			let mut item_map : HashMap<String, String> = HashMap::new();
			item_map.insert("name" .to_string(), s.name.to_string());
			item_map.insert("start".to_string(), s.start.format("%Y-%m-%d %H:%M:%S").to_string());
			item_map.insert("end"  .to_string(), s.end  .format("%Y-%m-%d %H:%M:%S").to_string());
			data_to_save.push(item_map);
		}
		save_json(&filename, &data_to_save);
	}


	// // List of segments
	// let mut _out = list_of_segments(&PATH_ROOT_FOLDER.to_string());
	// _out.sort();

	// Execute python script
	// let output_command = Command::new("pythonw")
	let output_command = Command::new("C:/Users/Cherrypie/AppData/Local/Programs/Python/Python39/pythonw.exe")
		.arg("C:/Github/Apuntes/Windows_automated/personal_tracking_export_pomodoros.py")
		.spawn();
	if output_command.is_err() { println!("Python script gave an error: {:?}", output_command.err().unwrap()); }
	else {                       println!("Python script executed"); }

	// We get the unique names of the segments
	let mut unique_names : Vec<String> = Vec::new();
	for i in all_segments.iter().rev() {
		if !unique_names.contains(&i.name) {
			unique_names.push(i.name.clone());
		}
	}
	unique_names

}

#[tauri::command]
fn conf_get_time_pomodoro_in_min() -> f32 {

	#[cfg(debug_assertions)]
	{
		1.2
	}

	#[cfg(not(debug_assertions))]
	{
		25.0
	}
}

#[tauri::command]
fn get_last_date_of_segment() -> String {

	let mut last_date : String = "".to_string();

	let mut list_of_files : Vec<Segment> = list_of_segments(&PATH_ROOT_FOLDER.to_string());
	list_of_files.sort();

	if list_of_files.len() > 0
	{
		let last_file = list_of_files.last().unwrap();
		// let last_file = last_file.split(".").collect::<Vec<&str>>()[0];
		// last_date = last_file.to_string();
		last_date = last_file.end.to_string();
	}

	last_date
}

// static PATH_ROOT_FOLDER : &str = "C:/data_pomodoros";
const PATH_ROOT_FOLDER : &str = "C:/Registries/Pomodoro";

fn main() {

	#[cfg(debug_assertions)] { println!("Starting backend..."); }

	// Create path with intermediate folders if it doesn't exist
	if !Path::new(&PATH_ROOT_FOLDER).exists() {
		#[cfg(debug_assertions)] { println!("Created missing path..."); }
		create_dir_all(&PATH_ROOT_FOLDER).unwrap();
	}


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
		])
		.run(tauri::generate_context!())
		.expect("error while running tauri application");

}
