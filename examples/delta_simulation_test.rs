// Proper Melda Usage Demonstration
// Shows the correct read-modify-write pattern for adding items to arrays

use melda::{filesystemadapter::FilesystemAdapter, melda::Melda};
use serde_json::json;
use std::fs;
use std::io;
use std::path::Path;
use std::sync::{Arc, RwLock};

fn main() {
    // Clean up
    _ = fs::remove_dir_all("delta_sim_alice");
    _ = fs::remove_dir_all("delta_sim_bob");
    
    println!("=== Proper Melda Usage Demonstration ===");
    println!("Showing the correct read-modify-write pattern");
    println!("for making incremental changes to documents\n");
    
    // Start with initial array
    let initial_json = json!({
        "doc": "Delta Simulation",
        "items♭": [
            {"_id": "item_1", "content": "Original item 1"},
            {"_id": "item_2", "content": "Original item 2"}, 
            {"_id": "item_3", "content": "Original item 3"}
        ]
    });
    
    // Initialize Alice
    let adapter_alice = Box::new(FilesystemAdapter::new("delta_sim_alice").unwrap());
    let mut melda_alice = Melda::new(Arc::new(RwLock::new(adapter_alice))).unwrap();
    melda_alice.update(initial_json.as_object().unwrap().clone()).unwrap();
    melda_alice.commit(Some(json!({"op": "initial_state"}).as_object().unwrap().clone())).unwrap();
    
    println!("Initial state:");
    let initial_state = melda_alice.read(None).unwrap();
    let initial_items = initial_state["items♭"].as_array().unwrap();
    for (i, item) in initial_items.iter().enumerate() {
        println!("  [{}] {}", i, item["_id"].as_str().unwrap());
    }
    println!();
    
    // Copy to Bob
    copy_recursively("delta_sim_alice", "delta_sim_bob").unwrap();
    let adapter_bob = Box::new(FilesystemAdapter::new("delta_sim_bob").unwrap());
    let melda_bob = Melda::new(Arc::new(RwLock::new(adapter_bob))).unwrap();
    
    println!("🔥 PROPER USAGE: Read-Modify-Write Pattern\n");
    
    // Alice: Read current state, modify it, write back
    println!("Alice: Reading current state, adding item_4, writing back");
    let mut alice_current = melda_alice.read(None).unwrap();
    
    // Add item_4 to the existing items array
    if let Some(items) = alice_current.get_mut("items♭").and_then(|v| v.as_array_mut()) {
        items.push(json!({"_id": "item_4", "content": "Alice's new item"}));
        println!("  Current items: {:?}", items.iter().map(|i| i["_id"].as_str().unwrap()).collect::<Vec<_>>());
    }
    
    println!("  Updating with modified state (all 4 items)");
    melda_alice.update(alice_current.clone()).unwrap();
    melda_alice.commit(Some(json!({"op": "alice_add_item_4"}).as_object().unwrap().clone())).unwrap();
    
    println!("\nAlice's state after 'delta' operation:");
    let alice_after = melda_alice.read(None).unwrap();
    let alice_items = alice_after["items♭"].as_array().unwrap();
    for (i, item) in alice_items.iter().enumerate() {
        println!("  [{}] {}", i, item["_id"].as_str().unwrap());
    }
    
    // Bob: Also uses read-modify-write pattern
    println!("\nBob: Reading current state, adding item_5, writing back");
    let mut bob_current = melda_bob.read(None).unwrap();
    
    // Add item_5 to the existing items array
    if let Some(items) = bob_current.get_mut("items♭").and_then(|v| v.as_array_mut()) {
        items.push(json!({"_id": "item_5", "content": "Bob's new item"}));
        println!("  Current items: {:?}", items.iter().map(|i| i["_id"].as_str().unwrap()).collect::<Vec<_>>());
    }
    
    println!("  Updating with modified state (all 4 items)");
    melda_bob.update(bob_current.clone()).unwrap();
    melda_bob.commit(Some(json!({"op": "bob_add_item_5"}).as_object().unwrap().clone())).unwrap();
    
    println!("\nBob's state after 'delta' operation:");
    let bob_after = melda_bob.read(None).unwrap();
    let bob_items = bob_after["items♭"].as_array().unwrap();
    for (i, item) in bob_items.iter().enumerate() {
        println!("  [{}] {}", i, item["_id"].as_str().unwrap());
    }
    
    println!("\n{}", "=".repeat(60));
    println!("ANALYSIS OF PROPER USAGE:");
    println!("Expected behavior using read-modify-write pattern:");
    println!("  - Alice: [item_1, item_2, item_3, item_4]");
    println!("  - Bob:   [item_1, item_2, item_3, item_5]");
    println!("Actual behavior:");
    println!("  - Alice: {:?}", alice_items.iter().map(|i| i["_id"].as_str().unwrap()).collect::<Vec<_>>());
    println!("  - Bob:   {:?}", bob_items.iter().map(|i| i["_id"].as_str().unwrap()).collect::<Vec<_>>());
    
    if alice_items.len() == 4 && bob_items.len() == 4 {
        println!("\n✅ SUCCESS: Read-modify-write pattern works correctly!");
        println!("   All original items preserved, new items added as expected.");
        println!("   This is the proper way to use Melda's API.");
    } else {
        println!("\n❓ Unexpected result with read-modify-write pattern");
    }
    
    // Now let's see what happens when we merge the proper delta operations
    println!("\n🔄 MERGING WITH PROPER DELTA OPERATIONS:");
    let transferred_files = melda_alice.meld(&melda_bob).unwrap();
    println!("Files transferred: {:?}", transferred_files);
    println!("Number of files transferred: {}", transferred_files.len());
    
    let _ = melda_alice.refresh();
    
    let final_state = melda_alice.read(None).unwrap();
    let final_items = final_state["items♭"].as_array().unwrap();
    
    println!("\nFinal merged state:");
    for (i, item) in final_items.iter().enumerate() {
        println!("  [{}] {} - {}", i, 
            item["_id"].as_str().unwrap(),
            item["content"].as_str().unwrap());
    }
    
    println!("\n{}", "=".repeat(60));
    println!("FINAL ANALYSIS:");
    let final_ids: Vec<&str> = final_items.iter().map(|i| i["_id"].as_str().unwrap()).collect();
    
    println!("✅ Contains all original items: {}", 
        ["item_1", "item_2", "item_3"].iter().all(|id| final_ids.contains(id)));
    println!("✅ Contains Alice's addition: {}", final_ids.contains(&"item_4"));
    println!("✅ Contains Bob's addition: {}", final_ids.contains(&"item_5"));
    println!("✅ Total items: {}", final_items.len());
    
    if final_items.len() == 5 && 
       ["item_1", "item_2", "item_3"].iter().all(|id| final_ids.contains(id)) &&
       final_ids.contains(&"item_4") && 
       final_ids.contains(&"item_5") {
        println!("\n🎉 PERFECT! Read-modify-write pattern enables true delta behavior!");
        println!("   - All original items preserved");
        println!("   - Both additions merged correctly"); 
        println!("   - Only {} files transferred during sync", transferred_files.len());
        println!("   - This demonstrates Melda's delta-state CRDT nature!");
    }
    
    // Keep files for inspection - comment to clean up
    // _ = fs::remove_dir_all("delta_sim_alice");
    // _ = fs::remove_dir_all("delta_sim_bob");
    println!("\nFiles kept in delta_sim_alice/ and delta_sim_bob/ for inspection");
}

fn copy_recursively(source: impl AsRef<Path>, destination: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let filetype = entry.file_type()?;
        if filetype.is_dir() {
            copy_recursively(entry.path(), destination.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), destination.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}