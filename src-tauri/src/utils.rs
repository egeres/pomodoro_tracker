use chrono::DateTime;
use chrono::Local;
use chrono::NaiveDateTime;
use chrono::TimeZone;
use std::collections::HashMap;
use std::error::Error;
use std::fs::*;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration; // For using datetime_from_str

use crate::segment::Segment;
use crate::RUNNING;
use crate::START_TIME;
use crate::TIMER_TOTAL_S;

pub fn save_json(filename: &String, data_to_save: &Vec<HashMap<String, String>>) {
    #[cfg(debug_assertions)]
    {
        println!("Saving {}...", filename);
    }

    let mut file = match File::create(filename) {
        Ok(file) => file,
        Err(e) => {
            println!("Error creating file: {:?}", e);
            return;
        }
    };
    let json_data = serde_json::to_string_pretty(&data_to_save).unwrap();
    let out = file.write_all(json_data.as_bytes());

    #[cfg(debug_assertions)]
    {
        match out {
            Ok(_) => println!("File saved successfully"),
            Err(e) => println!("Error saving file: {:?}", e),
        }
    }
}

/// Parses a JSON file into a list of string-keyed maps.
///
/// This never panics: an unreadable file, invalid JSON, or a non-array root all
/// return `Err` so the caller can skip the whole file. Within the array, entries
/// that aren't objects are skipped, and only string-valued fields are kept (extra
/// non-string fields such as numbers are ignored).
pub fn load_json(file_name: &str) -> Result<Vec<HashMap<String, String>>, String> {
    if !Path::new(file_name).exists() {
        return Err(format!("'{}' does not exist", file_name));
    }

    #[cfg(debug_assertions)]
    {
        println!("Reading... {}", file_name);
    }

    let mut file = File::open(file_name).map_err(|e| format!("could not open file: {:?}", e))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| format!("could not read file: {:?}", e))?;

    let json_data: serde_json::Value =
        serde_json::from_str(&contents).map_err(|e| format!("invalid JSON: {:?}", e))?;

    let json_array = match json_data.as_array() {
        Some(arr) => arr,
        None => return Err("expected a top-level JSON array".to_string()),
    };

    let mut json_data_vec: Vec<HashMap<String, String>> = Vec::new();
    for item in json_array {
        // Non-object entries are skipped instead of crashing.
        let item = match item.as_object() {
            Some(obj) => obj,
            None => continue,
        };
        let mut item_map: HashMap<String, String> = HashMap::new();
        for (key, value) in item {
            // Only string values are kept; extra non-string fields are ignored.
            if let Some(s) = value.as_str() {
                item_map.insert(key.to_string(), s.to_string());
            }
        }
        json_data_vec.push(item_map);
    }
    Ok(json_data_vec)
}

/// Parses a datetime string, accepting both the format this app writes
/// (`%Y-%m-%d %H:%M:%S`) and RFC3339/ISO8601 timestamps with an offset
/// (e.g. `2026-05-31T10:58:18.549880+00:00`). Returns `None` if neither matches.
fn parse_datetime(value: &str) -> Option<DateTime<Local>> {
    if let Ok(naive) = NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S") {
        if let Some(dt) = Local.from_local_datetime(&naive).single() {
            return Some(dt);
        }
    }

    if let Ok(dt) = DateTime::parse_from_rfc3339(value) {
        return Some(dt.with_timezone(&Local));
    }

    None
}

// List files in a folder
pub fn list_files_in_folder(folder_name: &str) -> Result<Vec<String>, String> {
    if !Path::new(folder_name).exists() {
        return Err(format!("'{}' does not exist", folder_name));
    }

    let mut files: Vec<String> = Vec::new();
    let entries = read_dir(folder_name).map_err(|e| format!("could not read dir: {:?}", e))?;
    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        if let Some(file_name) = entry.path().file_name().and_then(|n| n.to_str()) {
            files.push(file_name.to_string());
        }
    }
    Ok(files)
}

// Create a list of segments with the info of the jsons in the data folder
pub fn list_of_segments(path_dir: &String) -> Vec<Segment> {
    let mut to_return: Vec<Segment> = Vec::new();

    // A missing/unreadable folder yields no segments instead of crashing.
    let files = match list_files_in_folder(path_dir) {
        Ok(files) => files,
        Err(e) => {
            eprintln!("Could not list '{}': {}", path_dir, e);
            return to_return;
        }
    };

    // Filter only files that end with .json
    let files: Vec<String> = files.into_iter().filter(|x| x.ends_with(".json")).collect();

    for file_name in files.iter() {
        let full_path = format!("{}/{}", path_dir, file_name);

        // Malformed or unreadable JSON files are skipped entirely.
        let items = match load_json(&full_path) {
            Ok(items) => items,
            Err(e) => {
                eprintln!("Skipping '{}': {}", full_path, e);
                continue;
            }
        };

        for (idx, i) in items.iter().enumerate() {
            // "name", "start" and "end" are required. If any is missing or can't
            // be parsed, we skip just this pomodoro rather than crashing.
            let name = match i.get("name") {
                Some(name) => name.to_string(),
                None => {
                    eprintln!(
                        "Skipping entry #{} in '{}': missing required field 'name'",
                        idx, full_path
                    );
                    continue;
                }
            };

            let start = match i.get("start") {
                None => {
                    eprintln!(
                        "Skipping entry #{} in '{}': missing required field 'start'",
                        idx, full_path
                    );
                    continue;
                }
                Some(raw) => match parse_datetime(raw) {
                    Some(start) => start,
                    None => {
                        eprintln!(
                            "Skipping entry #{} in '{}': unparseable 'start' value '{}'",
                            idx, full_path, raw
                        );
                        continue;
                    }
                },
            };

            let end = match i.get("end") {
                None => {
                    eprintln!(
                        "Skipping entry #{} in '{}': missing required field 'end'",
                        idx, full_path
                    );
                    continue;
                }
                Some(raw) => match parse_datetime(raw) {
                    Some(end) => end,
                    None => {
                        eprintln!(
                            "Skipping entry #{} in '{}': unparseable 'end' value '{}'",
                            idx, full_path, raw
                        );
                        continue;
                    }
                },
            };

            // Every other field is optional. "type" is the current key; the older
            // "type_of_event" key is still accepted for backwards compatibility.
            let type_of_event = i
                .get("type")
                .or_else(|| i.get("type_of_event"))
                .map(|x| x.to_string());

            to_return.push(Segment {
                name,
                start,
                end,
                type_of_event,
                uuid: i.get("uuid").map(|x| x.to_string()),
                os_login: i.get("os.login()").map(|x| x.to_string()),
                platform: i.get("platform.system()").map(|x| x.to_string()),
                machine_name: i.get("machine_name").map(|x| x.to_string()),
                generated_by: i.get("generated_by").map(|x| x.to_string()),
                way_this_info_was_added: i.get("way_this_info_was_added").map(|x| x.to_string()),
                datetime_of_annotation: i.get("datetime_of_annotation").map(|x| x.to_string()),
            });
        }
    }

    println!("File count: {}", files.len());
    println!("Pomodoro count: {}", to_return.len());

    // WIP, sort this vector once ord is implemented in Segment

    return to_return;
}

pub fn execute_script_python(script_to_execute: &str) -> Result<(), Box<dyn Error>> {
    println!("{}", script_to_execute);

    // We check the script exists
    if Path::new(script_to_execute).exists() {
        let output_command = Command::new("pythonw").arg(script_to_execute).spawn();
        if output_command.is_err() {
            println!(
                "Python script gave an error: {:?}",
                output_command.err().unwrap()
            );
        } else {
            println!("Python script executed successfully");
        }
    } else {
        println!("File does not exist: {}", script_to_execute);
    }

    Ok(())
}

/// Starts a loop that writes 0.0 or 1.0 in the designated file to indicate the progress
/// of the pomodoro
pub fn filewritter(file_name: &str) {
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

            if *RUNNING.lock().unwrap() {
                let time_start: chrono::DateTime<Local> = *START_TIME.lock().unwrap();
                let time_since_start_time_in_seconds: i64 =
                    (time_start - Local::now()).num_seconds();

                let a =
                    time_since_start_time_in_seconds as f64 / *TIMER_TOTAL_S.lock().unwrap() as f64;

                let v = f64::max(0.0 as f64, a);

                file.write_all(v.to_string().as_bytes()).unwrap();
            } else {
                file.write_all(b"1.00").unwrap();
            }
        }
    }
}
