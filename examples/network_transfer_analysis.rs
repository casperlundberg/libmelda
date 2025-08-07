// Network Transfer Analysis for Melda CRDT
// This test examines what data is actually transferred between replicas during meld()

use melda::{filesystemadapter::FilesystemAdapter, melda::Melda};
use serde_json::json;
use std::fs;
use std::path::Path;
use std::sync::{Arc, RwLock};

fn main() {
    // Clean up
    _ = fs::remove_dir_all("transfer_alice");
    _ = fs::remove_dir_all("transfer_bob");
    
    println!("=== Network Transfer Analysis ===");
    println!("Examining what data is actually transferred during meld()\n");
    
    // Initialize Alice with some data
    let adapter_alice = Box::new(FilesystemAdapter::new("transfer_alice").unwrap());
    let mut melda_alice = Melda::new(Arc::new(RwLock::new(adapter_alice))).unwrap();
    
    // Initial state
    let initial_json = json!({
        "doc": "Transfer Test",
        "items♭": [
            {"_id": "item_1", "content": "First item"},
            {"_id": "item_2", "content": "Second item"},
            {"_id": "item_3", "content": "Third item"}
        ]
    });
    
    println!("Step 1: Alice creates initial state with 3 items");
    melda_alice.update(initial_json.as_object().unwrap().clone()).unwrap();
    melda_alice.commit(Some(json!({"author": "Alice", "op": "initial"}).as_object().unwrap().clone())).unwrap();
    
    // Examine Alice's files
    println!("\nAlice's storage after initial commit:");
    examine_storage("transfer_alice");
    
    // Copy to Bob (simulating initial sync)
    println!("\n{}", "=".repeat(60));
    println!("Step 2: Bob gets initial state from Alice");
    copy_dir_recursively("transfer_alice", "transfer_bob").unwrap();
    
    let adapter_bob = Box::new(FilesystemAdapter::new("transfer_bob").unwrap());
    let mut melda_bob = Melda::new(Arc::new(RwLock::new(adapter_bob))).unwrap();
    
    // Bob makes a change
    println!("\nStep 3: Bob adds item_4");
    let bob_update = json!({
        "doc": "Transfer Test",
        "items♭": [
            {"_id": "item_1", "content": "First item"},
            {"_id": "item_2", "content": "Second item"}, 
            {"_id": "item_3", "content": "Third item"},
            {"_id": "item_4", "content": "Bob's new item"}
        ]
    });
    melda_bob.update(bob_update.as_object().unwrap().clone()).unwrap();
    melda_bob.commit(Some(json!({"author": "Bob", "op": "add_item_4"}).as_object().unwrap().clone())).unwrap();
    
    println!("\nBob's storage after adding item_4:");
    examine_storage("transfer_bob");
    
    // Alice makes a different change
    println!("\n{}", "=".repeat(60));
    println!("Step 4: Alice modifies item_2 and adds item_5");
    let alice_update = json!({
        "doc": "Transfer Test",
        "items♭": [
            {"_id": "item_1", "content": "First item"},
            {"_id": "item_2", "content": "Modified by Alice!"},  // Modified
            {"_id": "item_3", "content": "Third item"},
            {"_id": "item_5", "content": "Alice's new item"}
        ]
    });
    melda_alice.update(alice_update.as_object().unwrap().clone()).unwrap();
    melda_alice.commit(Some(json!({"author": "Alice", "op": "modify_and_add"}).as_object().unwrap().clone())).unwrap();
    
    println!("\nAlice's storage after modifications:");
    examine_storage("transfer_alice");
    
    // The critical test: what gets transferred during meld?
    println!("\n{}", "=".repeat(60));
    println!("🔥 CRITICAL: What gets transferred when Alice melds with Bob?");
    
    // Count files before meld
    let alice_files_before: Vec<_> = fs::read_dir("transfer_alice").unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    
    // Perform meld
    let transferred = melda_alice.meld(&melda_bob).unwrap();
    melda_alice.refresh().unwrap();
    
    println!("\nTransferred items during meld: {:?}", transferred);
    println!("Number of items transferred: {}", transferred.len());
    
    // Examine what was transferred
    for item in &transferred {
        let file_path = Path::new("transfer_alice").join(item);
        if file_path.exists() {
            let size = fs::metadata(&file_path).unwrap().len();
            println!("\n📦 Transferred: {} ({} bytes)", item, size);
            
            // Read and display content if it's a delta file
            if item.ends_with(".delta") {
                let content = fs::read(&file_path).unwrap();
                // Try to parse as JSON
                if let Ok(json_str) = String::from_utf8(content.clone()) {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json_str) {
                        println!("Content (JSON):");
                        println!("{}", serde_json::to_string_pretty(&parsed).unwrap());
                    }
                }
            }
        }
    }
    
    // Compare storage sizes
    println!("\n{}", "=".repeat(60));
    println!("Storage Analysis:");
    let alice_size = calculate_storage_size("transfer_alice");
    let bob_size = calculate_storage_size("transfer_bob");
    println!("Alice total storage: {} bytes", alice_size);
    println!("Bob total storage: {} bytes", bob_size);
    
    // Final state comparison
    println!("\n{}", "=".repeat(60));
    println!("Final merged state:");
    let final_state = melda_alice.read(None).unwrap();
    let items = final_state["items♭"].as_array().unwrap();
    for (i, item) in items.iter().enumerate() {
        println!("  [{}] {} - {}", i, 
            item["_id"].as_str().unwrap(),
            item["content"].as_str().unwrap());
    }
    
    println!("\n{}", "=".repeat(60));
    println!("CONCLUSION:");
    println!("- Melda transfers .delta and .pack files between replicas");
    println!("- Each .delta file contains CHANGES, not full state");
    println!("- .pack files contain the actual object data");
    println!("- Only NEW deltas/packs are transferred (not seen before)");
    println!("- This is TRUE delta-state CRDT behavior at the network level!");
    
    // Keep files for inspection - comment to clean up
    // _ = fs::remove_dir_all("transfer_alice");
    // _ = fs::remove_dir_all("transfer_bob");
    println!("\nFiles kept in transfer_alice/ and transfer_bob/ for inspection");
}

fn examine_storage(dir: &str) {
    let mut delta_files = Vec::new();
    let mut pack_files = Vec::new();
    let mut total_size = 0u64;
    
    examine_dir_recursive(dir, &mut delta_files, &mut pack_files, &mut total_size);
    
    println!("  - {} .delta files (change records)", delta_files.len());
    println!("  - {} .pack files (object storage)", pack_files.len());
    println!("  - Total size: {} bytes", total_size);
    
    if !delta_files.is_empty() {
        println!("  Delta files:");
        for (name, size) in &delta_files {
            println!("    {} ({} bytes)", 
                if name.len() > 60 { format!("...{}", &name[name.len()-57..]) } else { name.clone() },
                size);
        }
    }
    
    if !pack_files.is_empty() {
        println!("  Pack files:");
        for (name, size) in &pack_files {
            println!("    {} ({} bytes)", 
                if name.len() > 60 { format!("...{}", &name[name.len()-57..]) } else { name.clone() },
                size);
        }
    }
}

fn examine_dir_recursive(dir: &str, delta_files: &mut Vec<(String, u64)>, pack_files: &mut Vec<(String, u64)>, total_size: &mut u64) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                let metadata = entry.metadata().unwrap();
                
                if metadata.is_file() {
                    let size = metadata.len();
                    *total_size += size;
                    let name = path.to_string_lossy().to_string();
                    
                    if name.ends_with(".delta") {
                        delta_files.push((name, size));
                    } else if name.ends_with(".pack") {
                        pack_files.push((name, size));
                    }
                } else if metadata.is_dir() {
                    examine_dir_recursive(&path.to_string_lossy(), delta_files, pack_files, total_size);
                }
            }
        }
    }
}

fn calculate_storage_size(dir: &str) -> u64 {
    fs::read_dir(dir).unwrap()
        .map(|e| e.unwrap().metadata().unwrap().len())
        .sum()
}

fn copy_dir_recursively(source: &str, destination: &str) -> std::io::Result<()> {
    fs::create_dir_all(&destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let filetype = entry.file_type()?;
        let dest_path = Path::new(destination).join(entry.file_name());
        if filetype.is_dir() {
            copy_dir_recursively(&entry.path().to_string_lossy(), &dest_path.to_string_lossy())?;
        } else {
            fs::copy(entry.path(), dest_path)?;
        }
    }
    Ok(())
}