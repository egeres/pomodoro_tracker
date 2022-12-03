
#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

// launch dev mode with:
// cargo tauri dev

use std::fs::*;

use chrono::{DateTime, Local}; // TimeZone, NaiveDateTime
use std::path::Path;
// use std::cmp::Ordering;

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

mod utils;











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
