// Proper Array Index Insertion Test for Melda CRDT  
// Tests actual array index positioning with pre-existing elements

use melda::{filesystemadapter::FilesystemAdapter, melda::Melda};
use serde_json::json;
use std::fs;
use std::io;
use std::path::Path;
use std::sync::{Arc, RwLock};

fn main() {
    // Clean up
    _ = fs::remove_dir_all("index_test_r1");
    _ = fs::remove_dir_all("index_test_r2");
    _ = fs::remove_dir_all("index_test_r3");
    
    println!("=== Proper Array Index Insertion Test ===");
    println!("Initial: [item_0, item_1, item_2, item_3]");
    println!("R1: Insert 'NEW_A' at index 1 (between item_0 and item_1)");
    println!("R2: Insert 'NEW_B' at index 2 (between item_1 and item_2)");  
    println!("R3: Insert 'NEW_C' at index 3 (between item_2 and item_3)");
    println!("Expected: [item_0, NEW_A, item_1, NEW_B, item_2, NEW_C, item_3]\n");
    
    // Start with 4 existing elements
    let initial_json = json!({
        "doc": "Index Test", 
        "items♭": [
            {"_id": "item_0", "content": "Original item 0"},
            {"_id": "item_1", "content": "Original item 1"}, 
            {"_id": "item_2", "content": "Original item 2"},
            {"_id": "item_3", "content": "Original item 3"}
        ]
    });
    
    // Initialize R1
    let adapter_r1 = Box::new(FilesystemAdapter::new("index_test_r1").unwrap());
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
    copy_recursively("index_test_r1", "index_test_r2").unwrap();
    copy_recursively("index_test_r1", "index_test_r3").unwrap();
    
    let adapter_r2 = Box::new(FilesystemAdapter::new("index_test_r2").unwrap());
    let melda_r2 = Melda::new(Arc::new(RwLock::new(adapter_r2))).unwrap();
    
    let adapter_r3 = Box::new(FilesystemAdapter::new("index_test_r3").unwrap());
    let melda_r3 = Melda::new(Arc::new(RwLock::new(adapter_r3))).unwrap();
    
    // R1: Insert NEW_A at index 1 (after item_0, before item_1)
    let r1_update = json!({
        "doc": "Index Test",
        "items♭": [
            {"_id": "item_0", "content": "Original item 0"},
            {"_id": "NEW_A", "content": "R1 inserted at index 1"},
            {"_id": "item_1", "content": "Original item 1"}, 
            {"_id": "item_2", "content": "Original item 2"},
            {"_id": "item_3", "content": "Original item 3"}
        ]
    });
    melda_r1.update(r1_update.as_object().unwrap().clone()).unwrap();
    melda_r1.commit(Some(json!({"op": "r1_insert_index_1"}).as_object().unwrap().clone())).unwrap();
    
    // R2: Insert NEW_B at index 2 (after item_1, before item_2) 
    let r2_update = json!({
        "doc": "Index Test",
        "items♭": [
            {"_id": "item_0", "content": "Original item 0"},
            {"_id": "item_1", "content": "Original item 1"},
            {"_id": "NEW_B", "content": "R2 inserted at index 2"},
            {"_id": "item_2", "content": "Original item 2"},
            {"_id": "item_3", "content": "Original item 3"}
        ]
    });
    melda_r2.update(r2_update.as_object().unwrap().clone()).unwrap();
    melda_r2.commit(Some(json!({"op": "r2_insert_index_2"}).as_object().unwrap().clone())).unwrap();
    
    // R3: Insert NEW_C at index 3 (after item_2, before item_3)
    let r3_update = json!({
        "doc": "Index Test", 
        "items♭": [
            {"_id": "item_0", "content": "Original item 0"},
            {"_id": "item_1", "content": "Original item 1"}, 
            {"_id": "item_2", "content": "Original item 2"},
            {"_id": "NEW_C", "content": "R3 inserted at index 3"},
            {"_id": "item_3", "content": "Original item 3"}
        ]
    });
    melda_r3.update(r3_update.as_object().unwrap().clone()).unwrap();
    melda_r3.commit(Some(json!({"op": "r3_insert_index_3"}).as_object().unwrap().clone())).unwrap();
    
    // Show local states before sync
    println!("Local states after insertions:");
    let r1_local = melda_r1.read(None).unwrap();
    let r1_items = r1_local["items♭"].as_array().unwrap();
    println!("R1 local:");
    for (i, item) in r1_items.iter().enumerate() {
        println!("  [{}] {}", i, item["_id"].as_str().unwrap());
    }
    
    let r2_local = melda_r2.read(None).unwrap();
    let r2_items = r2_local["items♭"].as_array().unwrap();
    println!("R2 local:");
    for (i, item) in r2_items.iter().enumerate() {
        println!("  [{}] {}", i, item["_id"].as_str().unwrap());
    }
    
    let r3_local = melda_r3.read(None).unwrap();
    let r3_items = r3_local["items♭"].as_array().unwrap();
    println!("R3 local:");
    for (i, item) in r3_items.iter().enumerate() {
        println!("  [{}] {}", i, item["_id"].as_str().unwrap());
    }
    println!();
    
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
    let expected_order = vec!["item_0", "NEW_A", "item_1", "NEW_B", "item_2", "NEW_C", "item_3"];
    
    println!();
    if actual_order == expected_order {
        println!("✅ Perfect! Array insertions respected intended positions");
    } else {
        println!("❌ ARRAY POSITIONING NOT RESPECTED!");
        println!("Expected: {:?}", expected_order);
        println!("Actual:   {:?}", actual_order);
        
        // Analyze the difference
        println!("\nAnalysis:");
        println!("- All 7 elements present? {}", items.len() == 7);
        println!("- All original elements preserved? {}", 
            ["item_0", "item_1", "item_2", "item_3"].iter()
                .all(|id| actual_order.contains(id)));
        println!("- All insertions preserved? {}", 
            ["NEW_A", "NEW_B", "NEW_C"].iter()
                .all(|id| actual_order.contains(id)));
    }
    
    // Clean up
    _ = fs::remove_dir_all("index_test_r1");
    _ = fs::remove_dir_all("index_test_r2");
    _ = fs::remove_dir_all("index_test_r3");
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