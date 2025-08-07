// Delta Simulation Test for Melda CRDT
// Tests what happens when we try to simulate delta operations by only providing new elements

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
    
    println!("=== Delta Simulation Test ===");
    println!("Testing what happens when we simulate delta operations");
    println!("by only providing new elements (not full state)\n");
    
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
    
    println!("🔥 CRITICAL TEST: What happens when we simulate 'delta' operations?\n");
    
    // Alice tries to "add" item_4 by only providing the new item
    println!("Alice attempts to 'add' item_4 by providing only the new element:");
    let alice_delta_attempt = json!({
        "doc": "Delta Simulation",
        "items♭": [
            {"_id": "item_4", "content": "Alice's new item"}
        ]
    });
    
    println!("Alice's 'delta' update contains only: [item_4]");
    melda_alice.update(alice_delta_attempt.as_object().unwrap().clone()).unwrap();
    melda_alice.commit(Some(json!({"op": "alice_add_item_4"}).as_object().unwrap().clone())).unwrap();
    
    println!("\nAlice's state after 'delta' operation:");
    let alice_after = melda_alice.read(None).unwrap();
    let alice_items = alice_after["items♭"].as_array().unwrap();
    for (i, item) in alice_items.iter().enumerate() {
        println!("  [{}] {}", i, item["_id"].as_str().unwrap());
    }
    
    // Bob tries to "add" item_5 the same way
    println!("\nBob attempts to 'add' item_5 by providing only the new element:");
    let bob_delta_attempt = json!({
        "doc": "Delta Simulation",
        "items♭": [
            {"_id": "item_5", "content": "Bob's new item"}
        ]
    });
    
    println!("Bob's 'delta' update contains only: [item_5]");
    melda_bob.update(bob_delta_attempt.as_object().unwrap().clone()).unwrap();
    melda_bob.commit(Some(json!({"op": "bob_add_item_5"}).as_object().unwrap().clone())).unwrap();
    
    println!("\nBob's state after 'delta' operation:");
    let bob_after = melda_bob.read(None).unwrap();
    let bob_items = bob_after["items♭"].as_array().unwrap();
    for (i, item) in bob_items.iter().enumerate() {
        println!("  [{}] {}", i, item["_id"].as_str().unwrap());
    }
    
    println!("\n{}", "=".repeat(60));
    println!("ANALYSIS OF 'DELTA' SIMULATION:");
    println!("Expected behavior for true delta operations:");
    println!("  - Alice: [item_1, item_2, item_3, item_4]");
    println!("  - Bob:   [item_1, item_2, item_3, item_5]");
    println!("Actual behavior (state-based interpretation):");
    println!("  - Alice: {:?}", alice_items.iter().map(|i| i["_id"].as_str().unwrap()).collect::<Vec<_>>());
    println!("  - Bob:   {:?}", bob_items.iter().map(|i| i["_id"].as_str().unwrap()).collect::<Vec<_>>());
    
    if alice_items.len() == 1 && bob_items.len() == 1 {
        println!("\n❌ CONFIRMATION: Melda interpreted partial arrays as COMPLETE STATE REPLACEMENT");
        println!("   Original items [item_1, item_2, item_3] were DELETED because they weren't");
        println!("   included in the update. This confirms Melda is purely state-based.");
    } else {
        println!("\n❓ UNEXPECTED: Melda preserved some original items despite partial update");
    }
    
    // Now let's see what happens when we merge these "delta" operations
    println!("\n🔄 MERGING THE 'DELTA' OPERATIONS:");
    melda_alice.meld(&melda_bob).unwrap();
    let _ = melda_alice.refresh();
    
    let final_state = melda_alice.read(None).unwrap();
    let final_items = final_state["items♭"].as_array().unwrap();
    
    println!("Final merged state:");
    for (i, item) in final_items.iter().enumerate() {
        println!("  [{}] {}", i, item["_id"].as_str().unwrap());
    }
    
    println!("\nFinal Analysis:");
    let final_ids: Vec<&str> = final_items.iter().map(|i| i["_id"].as_str().unwrap()).collect();
    
    println!("- Contains original items? {}", 
        ["item_1", "item_2", "item_3"].iter().any(|id| final_ids.contains(id)));
    println!("- Contains Alice's addition? {}", final_ids.contains(&"item_4"));
    println!("- Contains Bob's addition? {}", final_ids.contains(&"item_5"));
    println!("- Total items: {}", final_items.len());
    
    if !["item_1", "item_2", "item_3"].iter().any(|id| final_ids.contains(id)) {
        println!("\n🚨 CRITICAL FINDING: All original items were permanently lost!");
        println!("   This proves that Melda requires COMPLETE state in every update.");
        println!("   There is no way to perform incremental 'delta' operations.");
    }
    
    // Clean up
    _ = fs::remove_dir_all("delta_sim_alice");
    _ = fs::remove_dir_all("delta_sim_bob");
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