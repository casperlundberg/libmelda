// True Delta Insertion Test for Melda CRDT
// Tests actual delta operations (adding single elements) rather than full state updates

use melda::{filesystemadapter::FilesystemAdapter, melda::Melda};
use serde_json::json;
use std::fs;
use std::io;
use std::path::Path;
use std::sync::{Arc, RwLock};

fn main() {
    // Clean up
    _ = fs::remove_dir_all("delta_test_r1");
    _ = fs::remove_dir_all("delta_test_r2");
    _ = fs::remove_dir_all("delta_test_r3");
    
    println!("=== True Delta Insertion Test ===");
    println!("This tests actual incremental delta operations, not full state updates");
    println!("Initial: [item_0, item_1, item_2, item_3]");
    println!("R1: Add 'NEW_A' (should try to insert at specific position)");
    println!("R2: Add 'NEW_B' (should try to insert at specific position)");  
    println!("R3: Add 'NEW_C' (should try to insert at specific position)");
    println!("Question: Can we control insertion position with delta operations?\n");
    
    // Start with 4 existing elements
    let initial_json = json!({
        "doc": "Delta Test", 
        "items♭": [
            {"_id": "item_0", "content": "Original item 0"},
            {"_id": "item_1", "content": "Original item 1"}, 
            {"_id": "item_2", "content": "Original item 2"},
            {"_id": "item_3", "content": "Original item 3"}
        ]
    });
    
    // Initialize R1
    let adapter_r1 = Box::new(FilesystemAdapter::new("delta_test_r1").unwrap());
    let mut melda_r1 = Melda::new(Arc::new(RwLock::new(adapter_r1))).unwrap();
    melda_r1.update(initial_json.as_object().unwrap().clone()).unwrap();
    melda_r1.commit(Some(json!({"init": true}).as_object().unwrap().clone())).unwrap();
    
    println!("Initial state:");
    let initial_state = melda_r1.read(None).unwrap();
    let initial_items = initial_state["items♭"].as_array().unwrap();
    for (i, item) in initial_items.iter().enumerate() {
        println!("  [{}] {}", i, item["_id"].as_str().unwrap());
    }
    println!();
    
    // Copy to R2 and R3
    copy_recursively("delta_test_r1", "delta_test_r2").unwrap();
    copy_recursively("delta_test_r1", "delta_test_r3").unwrap();
    
    let adapter_r2 = Box::new(FilesystemAdapter::new("delta_test_r2").unwrap());
    let melda_r2 = Melda::new(Arc::new(RwLock::new(adapter_r2))).unwrap();
    
    let adapter_r3 = Box::new(FilesystemAdapter::new("delta_test_r3").unwrap());
    let melda_r3 = Melda::new(Arc::new(RwLock::new(adapter_r3))).unwrap();
    
    // R1: Try to add NEW_A - but how do we specify WHERE in a delta operation?
    // Let's read current state and add to it
    let r1_current = melda_r1.read(None).unwrap();
    let mut r1_items = r1_current["items♭"].as_array().unwrap().clone();
    
    // Insert NEW_A at position 1 (between item_0 and item_1)
    r1_items.insert(1, json!({
        "_id": "NEW_A", 
        "content": "R1 inserted via delta at position 1"
    }));
    
    let r1_update = json!({
        "doc": "Delta Test",
        "items♭": r1_items
    });
    melda_r1.update(r1_update.as_object().unwrap().clone()).unwrap();
    melda_r1.commit(Some(json!({"op": "r1_add_NEW_A"}).as_object().unwrap().clone())).unwrap();
    
    // R2: Try to add NEW_B at position 2 (but relative to original, not R1's modified state)
    let r2_current = melda_r2.read(None).unwrap();
    let mut r2_items = r2_current["items♭"].as_array().unwrap().clone();
    
    // Insert NEW_B at position 2 (between item_1 and item_2 in original)
    r2_items.insert(2, json!({
        "_id": "NEW_B", 
        "content": "R2 inserted via delta at position 2"
    }));
    
    let r2_update = json!({
        "doc": "Delta Test",
        "items♭": r2_items
    });
    melda_r2.update(r2_update.as_object().unwrap().clone()).unwrap();
    melda_r2.commit(Some(json!({"op": "r2_add_NEW_B"}).as_object().unwrap().clone())).unwrap();
    
    // R3: Try to add NEW_C at position 3
    let r3_current = melda_r3.read(None).unwrap();
    let mut r3_items = r3_current["items♭"].as_array().unwrap().clone();
    
    // Insert NEW_C at position 3 (between item_2 and item_3 in original)
    r3_items.insert(3, json!({
        "_id": "NEW_C", 
        "content": "R3 inserted via delta at position 3"
    }));
    
    let r3_update = json!({
        "doc": "Delta Test",
        "items♭": r3_items
    });
    melda_r3.update(r3_update.as_object().unwrap().clone()).unwrap();
    melda_r3.commit(Some(json!({"op": "r3_add_NEW_C"}).as_object().unwrap().clone())).unwrap();
    
    // Show local states before sync
    println!("Local states after delta insertions:");
    
    let r1_local = melda_r1.read(None).unwrap();
    let r1_items = r1_local["items♭"].as_array().unwrap();
    println!("R1 local (inserted NEW_A at pos 1):");
    for (i, item) in r1_items.iter().enumerate() {
        println!("  [{}] {}", i, item["_id"].as_str().unwrap());
    }
    
    let r2_local = melda_r2.read(None).unwrap();
    let r2_items = r2_local["items♭"].as_array().unwrap();
    println!("R2 local (inserted NEW_B at pos 2):");
    for (i, item) in r2_items.iter().enumerate() {
        println!("  [{}] {}", i, item["_id"].as_str().unwrap());
    }
    
    let r3_local = melda_r3.read(None).unwrap();
    let r3_items = r3_local["items♭"].as_array().unwrap();
    println!("R3 local (inserted NEW_C at pos 3):");
    for (i, item) in r3_items.iter().enumerate() {
        println!("  [{}] {}", i, item["_id"].as_str().unwrap());
    }
    println!();
    
    // Now the critical test: what happens when we merge?
    println!("🔥 CRITICAL TEST: Merging delta operations...\n");
    
    // Sync all
    melda_r1.meld(&melda_r2).unwrap();
    let _ = melda_r1.refresh();
    melda_r1.meld(&melda_r3).unwrap(); 
    let _ = melda_r1.refresh();
    
    let final_state = melda_r1.read(None).unwrap();
    let items = final_state["items♭"].as_array().unwrap();
    
    println!("Final merged state:");
    for (i, item) in items.iter().enumerate() {
        println!("  [{}] {}", i, item["_id"].as_str().unwrap());
    }
    
    let actual_order: Vec<&str> = items.iter()
        .map(|item| item["_id"].as_str().unwrap())
        .collect();
    
    // What would we ideally want?
    let ideal_order = vec!["item_0", "NEW_A", "item_1", "NEW_B", "item_2", "NEW_C", "item_3"];
    // What do we expect Melda's algorithm to produce?
    
    println!();
    println!("Analysis:");
    println!("- All 7 elements present? {}", items.len() == 7);
    println!("- All original elements preserved? {}", 
        ["item_0", "item_1", "item_2", "item_3"].iter()
            .all(|id| actual_order.contains(id)));
    println!("- All insertions preserved? {}", 
        ["NEW_A", "NEW_B", "NEW_C"].iter()
            .all(|id| actual_order.contains(id)));
    
    if actual_order == ideal_order {
        println!("\n✅ PERFECT! Delta insertions preserved intended positions");
    } else {
        println!("\n❓ DELTA MERGE BEHAVIOR:");
        println!("Ideal:  {:?}", ideal_order);
        println!("Actual: {:?}", actual_order);
        println!("This shows how Melda's merge algorithm handles concurrent delta insertions");
    }
    
    // Clean up
    _ = fs::remove_dir_all("delta_test_r1");
    _ = fs::remove_dir_all("delta_test_r2");
    _ = fs::remove_dir_all("delta_test_r3");
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