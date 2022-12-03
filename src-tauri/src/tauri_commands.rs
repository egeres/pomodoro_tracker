


use std::fs::*;
use std::collections::HashMap;
use std::path::Path;
use chrono::{Local}; // TimeZone, NaiveDateTime

use crate::utils::execute_script_python;
use crate::utils::list_of_segments;
use crate::utils::save_json;
use crate::Segment;
use crate::PATH_ROOT_FOLDER;
use crate::START_TIME;
use crate::TIMER_TOTAL_S;
use crate::RUNNING;



#[tauri::command]
pub fn command_retrieve_last_pomodoros() -> Vec<String> {

	// List of segments
	let a : String = PATH_ROOT_FOLDER.lock().unwrap().to_string();

	let mut _out = list_of_segments(&a.to_string());
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
pub fn annotate_pomodoro(pomodoro_name: String, duration_in_min:Option<i32>) -> Vec<String> {
	
	println!("dutati {}", duration_in_min.unwrap_or(999));

	let a : String = PATH_ROOT_FOLDER.lock().unwrap().to_string();

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
	if !Path::new(&a).exists() {
		create_dir_all(&a).unwrap();
	}

	let mut all_segments = list_of_segments(&a.to_string());
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
			let filename         : String                       = format!("{}/{}.json", a, current_stack_of_segments[0].start.format("%Y-%m-%d"));
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
			let filename         : String                       = format!("{}/{}.json", a, current_stack_of_segments[0].start.format("%Y-%m-%d"));
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
		let filename         : String                       = format!("{}/{}.json", a, current_stack_of_segments[0].start.format("%Y-%m-%d"));
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
	// let mut _out = list_of_segments(&a.to_string());
	// _out.sort();

	// Execute python script
	// let output_command = Command::new("pythonw")

	// let username = whoami::username();
	// let output_command = Command::new("pythonw")
	// 	.arg(format!("/Users/{}/.pomodoro/pomodoro.py", username))
	// 	.output()
	// 	.expect("failed to execute process");

	// let output_command = Command::new("C:/Users/Cherrypie/AppData/Local/Programs/Python/Python39/pythonw.exe")





	// let script_to_exdecute = "C:/Github/Apuntes/Windows_automated/personal_tracking_export_pomodoros.py";
	// // We check the script exists
	// if Path::new(script_to_exdecute).exists()
	// {
	// 	let output_command = Command::new("pythonw")
	// 		.arg(script_to_exdecute)
	// 		.spawn();
	// 	if output_command.is_err() { println!("Python script gave an error: {:?}", output_command.err().unwrap()); }
	// 	else {                       println!("Python script executed"); }
	// }
	// else
	// {
	// 	println!("File does not exist: {}", script_to_exdecute);
	// }





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
pub fn conf_get_time_pomodoro_in_min() -> f32 {
	let total_time_in_m = (*TIMER_TOTAL_S.lock().unwrap()) as f32 / 60.0;
	total_time_in_m
}

#[tauri::command]
pub fn get_last_date_of_segment() -> String {

	let mut last_date : String = "".to_string();
	let a : String = PATH_ROOT_FOLDER.lock().unwrap().to_string();

	let mut list_of_files : Vec<Segment> = list_of_segments(&a.to_string());
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

#[tauri::command]
pub fn pomodoro_start() {

	*START_TIME.lock().unwrap() = Local::now() + chrono::Duration::seconds(*TIMER_TOTAL_S.lock().unwrap() as i64);

	// This needs to be an option configurable by the user
	let o = execute_script_python("C:/Github/Apuntes/tool_blockdistractions_on.py");
	if  o.is_err() { println!("Error: {:?}", o.err().unwrap()); }

	*RUNNING.lock().unwrap() = true;

}

#[tauri::command]
pub fn pomodoro_end() {

	// This needs to be an option configurable by the user
	let o = execute_script_python("C:/Github/Apuntes/tool_blockdistractions_off.py");
	if  o.is_err() { println!("Error: {:?}", o.err().unwrap()); }
	let o = execute_script_python("C:/Github/Apuntes/Windows_automated/personal_tracking_export_pomodoros.py");
	if  o.is_err() { println!("Error: {:?}", o.err().unwrap()); }

	*RUNNING.lock().unwrap() = false;

}

#[tauri::command]
pub fn pomodoro_cancel() {
	*RUNNING.lock().unwrap() = false;
}
