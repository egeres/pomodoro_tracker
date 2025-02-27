#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use chrono::{DateTime, Local}; // TimeZone, NaiveDateTime
use lazy_static::lazy_static;
use std::fs::*;
use std::path::Path;
use std::sync::Mutex; // 1.4.0

mod segment;
use segment::Segment;
mod tauri_commands;
use tauri_commands::*;
mod server_http;
use server_http::start_httpserver;
mod utils;
use tauri::Manager;

use crate::utils::list_of_segments;

lazy_static! {
    static ref PATH_ROOT_FOLDER: Mutex<String> = Mutex::new("C:/Registries/Pomodoro".to_string());
    static ref PATH_DIR_LOCAL: Mutex<String> = Mutex::new("".to_string());
    static ref TIMER_TOTAL_S: Mutex<i32> = Mutex::new(2 * 60);
    static ref START_TIME: Mutex<DateTime<Local>> = Mutex::new(Local::now());
    static ref RUNNING: Mutex<bool> = Mutex::new(false);
    static ref CURRENT_SEGS: Mutex<Vec<Segment>> = Mutex::new(Vec::new());
}

fn main() {
    // We initialize variables
    #[cfg(debug_assertions)]
    {
        println!("Starting backend...");
        *TIMER_TOTAL_S.lock().unwrap() = 60 * 2;
    }
    #[cfg(not(debug_assertions))]
    {
        *TIMER_TOTAL_S.lock().unwrap() = 60 * 25;
    }

    {
        let a: String = PATH_ROOT_FOLDER.lock().unwrap().to_string();
        let mut _out = list_of_segments(&a.to_string());
        _out.sort();
        *CURRENT_SEGS.lock().unwrap() = _out;

        // For debugging n stuff
        // // Print the first 3
        // for i in 0..std::cmp::min(3, _out.len()) {
        // 	println!("{}: {:?}", i, _out[i]);
        // }
        // // Print the last 3
        // for i in std::cmp::max(0, _out.len() - 3).._out.len() {
        // 	println!("{}: {:?}", i, _out[i]);
        // }
    }
    // Print length of CURRENT_SEGS
    println!("CURRENT_SEGS len: {:?}", CURRENT_SEGS.lock().unwrap().len());

    // Folder where the executable is located
    // let new_path = std::env::current_exe().unwrap().parent().unwrap().to_str().unwrap().to_string();
    // *PATH_DIR_LOCAL.lock().unwrap() = new_path.clone();

    // let a = tauri::api::path::local_data_dir().unwrap();
    // *PATH_DIR_LOCAL.lock().unwrap() = a.to_str().unwrap().to_string();
    // let a = tauri::path::local_data_dir().unwrap();
    // let a = tauri::path::app_local_data_dir(&tauri::Config::default()).unwrap();
    // let a = tauri::path::app_data_dir(&tauri::Config::default()).unwrap();

    // let window = app.get_webview_window("main").unwrap();
    // window.open_devtools();

    // tauri::Builder::default()
    // .setup(|app| {
    //     // Access the local data directory
    //     let local_data_dir = app.path().app_local_data_dir().unwrap();
    //     // Use `local_data_dir` as needed
    //     *PATH_DIR_LOCAL.lock().unwrap() = local_data_dir.to_str().unwrap().to_string();
    //     println!("eee: {:?}", local_data_dir.as_path().to_string_lossy());
    //     println!("PATH_DIR_LOCAL: {:?}", *PATH_DIR_LOCAL.lock().unwrap());
    //     #[cfg(debug_assertions)]
    //     app.get_webview_window("main").unwrap().open_devtools();
    //     Ok(())
    // })
    // .run(tauri::generate_context!())
    // .expect("error while running tauri application");

    // *PATH_DIR_LOCAL.lock().unwrap() = a.to_str().unwrap().to_string();

    // #[cfg(debug_assertions)] {
    // println!("eee: {:?}", a.as_path().to_string_lossy());
    // println!("PATH_DIR_LOCAL: {:?}", *PATH_DIR_LOCAL.lock().unwrap());

    // let path_dir_local_exe_file : String =
    // }

    // Folder is created if it doesn't exist
    let a: String = PATH_ROOT_FOLDER.lock().unwrap().to_string();
    if !Path::new(&a).exists() {
        #[cfg(debug_assertions)]
        {
            println!("Created missing path...");
        }
        create_dir_all(&a).unwrap();
    }

    // File writter process
    // let path_dir_local = *PATH_DIR_LOCAL.lock().unwrap();
    // let path_fil_local: String = format!("{}/{}", &new_path, "current_state.txt");
    // let path_fil_local: String = format!(
    //     "{}/{}",
    //     *PATH_DIR_LOCAL.lock().unwrap(),
    //     "current_state_pomodoro.txt"
    // );
    // std::thread::spawn(move || utils::filewritter(&path_fil_local));

    // Server for http
    std::thread::spawn(start_httpserver);

    // Server for tauri
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {

            // Create console window in release mode
            // #[cfg(not(debug_assertions))]
            // {
            //     use std::fs::OpenOptions;
            //     use std::io::Write;
            //     // Allocate console in Windows release builds
            //     #[cfg(target_os = "windows")]
            //     unsafe {
            //         winapi::um::wincon::AttachConsole(winapi::um::wincon::ATTACH_PARENT_PROCESS);
            //     }
            // }

            // Access the local data directory
            let local_data_dir = app.path().app_local_data_dir().unwrap();
            // Use `local_data_dir` as needed
            *PATH_DIR_LOCAL.lock().unwrap() = local_data_dir.to_str().unwrap().to_string();
            println!("eee: {:?}", local_data_dir.as_path().to_string_lossy());
            println!("PATH_DIR_LOCAL: {:?}", *PATH_DIR_LOCAL.lock().unwrap());
            // #[cfg(debug_assertions)]
            // app.get_webview_window("main").unwrap().open_devtools();
            let path_fil_local: String = format!(
                "{}/{}",
                *PATH_DIR_LOCAL.lock().unwrap(),
                "current_state_pomodoro.txt"
            );
            std::thread::spawn(move || utils::filewritter(&path_fil_local));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            annotate_pomodoro,
            command_retrieve_last_pomodoros,
            conf_get_time_pomodoro_in_min,
            get_last_date_of_segment,
            pomodoro_cancel,
            pomodoro_end,
            pomodoro_start,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
