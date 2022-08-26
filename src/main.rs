#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

// Load all json content from a path
fn load_json(path: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
  let mut file = File::open(path)?;
  let mut contents = String::new();
  file.read_to_string(&mut contents)?;
  Ok(serde_json::from_str(&contents)?)
}

// Get unique keys from a json object
fn unique_keys_from_json(json: &serde_json::Value) -> Vec<String> {
  let mut keys = Vec::new();
  for (key, _) in json.as_object().unwrap() {
    if !keys.contains(&key.to_string()) {
      keys.push(key.to_string());
    }
  }
  keys
}

fn main() {
  tauri::Builder::default()
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
