
use std::fs::*;
use std::collections::HashMap;
use std::io::Read;
use std::io::Write;
use std::process::Command;
use std::error::Error;
use std::path::Path;
use std::thread;
use std::time::Duration;
use chrono::{Local}; // TimeZone, NaiveDateTime
use chrono::TimeZone; // For using datetime_from_str

use crate::segment::Segment;
use crate::TIMER_TOTAL_S;
use crate::START_TIME;
use crate::RUNNING;

pub fn save_json(filename:&String, data_to_save:&Vec<HashMap<String, String>>) {

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
		
		// to-do : refactor
		let a = i.get("type_of_event");
		let b: Option<String>  = match a {
			Some(x) => Some(x.to_string()),
			None    => None,
		};

		to_return.push(Segment {
			name         : i.get("name").unwrap_or(&"unknown".to_string()).to_string(),
			start        : Local.datetime_from_str(&i.get("start").unwrap(), "%Y-%m-%d %H:%M:%S").unwrap(),
			end          : Local.datetime_from_str(&i.get("end"  ).unwrap(), "%Y-%m-%d %H:%M:%S").unwrap(),
			type_of_event: b,
		});
	}

	// WIP, sort this vector once ord is implemented in Segment

	return to_return;
}

pub fn execute_script_python(script_to_execute : &str) -> Result<(), Box<dyn Error>> {
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

pub fn filewritter(file_name : &str) {

    let mut file_name_l = file_name.to_string();

    if !file_name_l.ends_with(".txt") {
        file_name_l = file_name_l + ".txt";
    }

    if !Path::new(file_name).exists() {
        File::create(file_name).unwrap();
    }

    // While true clear the file and write write "0.89" in the file each 5 seconds
    loop {

        thread::sleep(Duration::from_millis(1000));

        {
            let mut file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(&file_name_l)
                .unwrap();
            
            if *RUNNING.lock().unwrap()
            {
                let time_start: chrono::DateTime<Local> = *START_TIME.lock().unwrap();
                let time_since_start_time_in_seconds: i64 = (time_start - Local::now()).num_seconds();
                
                let a = time_since_start_time_in_seconds as f64 / *TIMER_TOTAL_S.lock().unwrap() as f64;

                let v = f64::max(0.0 as f64, a);

                file.write_all(
                    v.to_string().as_bytes()
                ).unwrap();
            }
            else
            {
                file.write_all(b"1.00").unwrap();
            }
        }
    }

}