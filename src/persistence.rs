use std::collections::HashMap;

/// Load approved messages from disk
pub fn load_approved_messages(file_path: &str) -> HashMap<u64, u64> {
    match std::fs::read_to_string(file_path) {
        Ok(data) => match serde_json::from_str::<HashMap<u64, u64>>(&data) {
            Ok(map) => {
                println!("Loaded {} approved messages from {}", map.len(), file_path);
                map
            }
            Err(e) => {
                println!(
                    "Failed to parse {}: {}. Starting with empty map.",
                    file_path, e
                );
                HashMap::new()
            }
        },
        Err(_) => {
            println!(
                "No persistence file found at {}. Starting with empty map.",
                file_path
            );
            HashMap::new()
        }
    }
}

/// Save approved messages to disk
pub fn save_approved_messages(file_path: &str, messages: &HashMap<u64, u64>) {
    match serde_json::to_string_pretty(messages) {
        Ok(data) => match std::fs::write(file_path, data) {
            Ok(_) => println!(
                "Saved {} approved messages to {}",
                messages.len(),
                file_path
            ),
            Err(e) => eprintln!("Failed to write to {}: {}", file_path, e),
        },
        Err(e) => eprintln!("Failed to serialize approved messages: {}", e),
    }
}
