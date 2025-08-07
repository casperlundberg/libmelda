// Test if Melda groups elements by replica origin

use melda::{filesystemadapter::FilesystemAdapter, melda::Melda};
use serde_json::json;
use std::fs;
use std::io;
use std::path::Path;
use std::sync::{Arc, RwLock};

fn main() {
    // Clean up
    _ = fs::remove_dir_all("group_test_r1");
    _ = fs::remove_dir_all("group_test_r2");
    _ = fs::remove_dir_all("group_test_r3");
    
    println!("=== Replica Grouping Test ===");
    println!("R1: Add 'a_item' and 'z_item'");
    println!("R2: Add 'b_item' and 'y_item'"); 
    println!("R3: Add 'c_item' and 'x_item'");
    println!("If replica-grouped: [a_item, z_item, b_item, y_item, c_item, x_item]");
    println!("If lexicographic: [a_item, b_item, c_item, x_item, y_item, z_item]\n");
    
    // Start with empty array
    let initial_json = json!({
        "doc": "Grouping Test",
        "items♭": []
    });
    
    // Initialize R1
    let adapter_r1 = Box::new(FilesystemAdapter::new("group_test_r1").unwrap());
    let mut melda_r1 = Melda::new(Arc::new(RwLock::new(adapter_r1))).unwrap();
    melda_r1.update(initial_json.as_object().unwrap().clone()).unwrap();
    melda_r1.commit(Some(json!({"init": true}).as_object().unwrap().clone())).unwrap();
    
    // Copy to R2 and R3
    copy_recursively("group_test_r1", "group_test_r2").unwrap();
    copy_recursively("group_test_r1", "group_test_r3").unwrap();
    
    let adapter_r2 = Box::new(FilesystemAdapter::new("group_test_r2").unwrap());
    let melda_r2 = Melda::new(Arc::new(RwLock::new(adapter_r2))).unwrap();
    
    let adapter_r3 = Box::new(FilesystemAdapter::new("group_test_r3").unwrap());
    let melda_r3 = Melda::new(Arc::new(RwLock::new(adapter_r3))).unwrap();
    
    // R1: Add 'a_item' and 'z_item'
    let r1_update = json!({
        "doc": "Grouping Test",
        "items♭": [
            {"_id": "a_item", "content": "A from R1"},
            {"_id": "z_item", "content": "Z from R1"}
        ]
    });
    melda_r1.update(r1_update.as_object().unwrap().clone()).unwrap();
    melda_r1.commit(Some(json!({"op": "r1_add"}).as_object().unwrap().clone())).unwrap();
    
    // R2: Add 'b_item' and 'y_item'  
    let r2_update = json!({
        "doc": "Grouping Test",
        "items♭": [
            {"_id": "b_item", "content": "B from R2"},
            {"_id": "y_item", "content": "Y from R2"}
        ]
    });
    melda_r2.update(r2_update.as_object().unwrap().clone()).unwrap();
    melda_r2.commit(Some(json!({"op": "r2_add"}).as_object().unwrap().clone())).unwrap();
    
    // R3: Add 'c_item' and 'x_item'
    let r3_update = json!({
        "doc": "Grouping Test", 
        "items♭": [
            {"_id": "c_item", "content": "C from R3"},
            {"_id": "x_item", "content": "X from R3"}
        ]
    });
    melda_r3.update(r3_update.as_object().unwrap().clone()).unwrap();
    melda_r3.commit(Some(json!({"op": "r3_add"}).as_object().unwrap().clone())).unwrap();
    
    // Sync all
    melda_r1.meld(&melda_r2).unwrap();
    let _ = melda_r1.refresh();
    melda_r1.meld(&melda_r3).unwrap(); 
    let _ = melda_r1.refresh();
    
    let final_state = melda_r1.read(None).unwrap();
    let items = final_state["items♭"].as_array().unwrap();
    
    println!("Actual order:");
    for (i, item) in items.iter().enumerate() {
        println!("  {}. {}", i, item["_id"].as_str().unwrap());
    }
    
    let actual_order: Vec<&str> = items.iter()
        .map(|item| item["_id"].as_str().unwrap())
        .collect();
    
    let replica_grouped = vec!["a_item", "z_item", "b_item", "y_item", "c_item", "x_item"];
    let lexicographic = vec!["a_item", "b_item", "c_item", "x_item", "y_item", "z_item"];
    
    if actual_order == replica_grouped {
        println!("\n✅ REPLICA GROUPING detected");
    } else if actual_order == lexicographic {
        println!("\n✅ LEXICOGRAPHIC ORDERING detected");  
    } else {
        println!("\n❓ UNKNOWN ORDERING PATTERN");
        println!("Expected replica-grouped: {:?}", replica_grouped);
        println!("Expected lexicographic:   {:?}", lexicographic);
        println!("Actual:                   {:?}", actual_order);
    }
    
    // Clean up
    _ = fs::remove_dir_all("group_test_r1");
    _ = fs::remove_dir_all("group_test_r2");
    _ = fs::remove_dir_all("group_test_r3");
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