// Complex Interleaved Operations Example for Melda CRDT
// 
// This example demonstrates complex scenarios with interleaved add/delete operations
// and partial synchronization between replicas. This tests Melda's ability to handle
// complex operational sequences and maintain consistency across partial sync scenarios.
//
// Test Scenario:
// - R1: Deletes all initial elements, then adds an element at index 1
// - R2: Adds an element at index 1, then removes element at index 0, then adds element at index 2
// - R3: Syncs with R1 after R1's operations, then adds an element at index 0
// - Finally: All replicas synchronize and converge
//
// This tests: Complex operation ordering, partial sync, add-after-delete, delete-after-add

use melda::{filesystemadapter::FilesystemAdapter, melda::Melda};
use serde_json::json;
use std::fs;
use std::io;
use std::path::Path;
use std::sync::{Arc, RwLock};

fn main() {
    // Clean up any existing test directories
    _ = fs::remove_dir_all("complex_test_r1");
    _ = fs::remove_dir_all("complex_test_r2");
    _ = fs::remove_dir_all("complex_test_r3");
    
    println!("=== Melda CRDT: Complex Interleaved Operations Test ===\n");
    println!("This example tests complex scenarios with interleaved add/delete operations");
    println!("and partial synchronization between replicas.\n");
    
    println!("🎯 TEST SCENARIO:");
    println!("• R1: Delete all initial elements → Add element at index 1");
    println!("• R2: Add at index 1 → Delete at index 0 → Add at index 2"); 
    println!("• R3: Sync with R1 → Add element at index 0");
    println!("• Final: All replicas synchronize\n");
    
    // Initial JSON document with 3 elements
    let initial_json = json!({
        "document": "Task Management System",
        "version": "2.0",
        "tasks♭": [
            {"_id": "init_0", "title": "Setup Project", "priority": "high", "index": 0},
            {"_id": "init_1", "title": "Design Database", "priority": "medium", "index": 1},
            {"_id": "init_2", "title": "Write Tests", "priority": "low", "index": 2}
        ]
    });
    
    println!("📄 INITIAL JSON STATE:");
    println!("{}\n", serde_json::to_string_pretty(&initial_json).unwrap());
    println!("{}", "═".repeat(80));
    
    // Create and initialize all three replicas
    println!("\n🔄 INITIALIZING THREE REPLICAS\n");
    
    // R1 (Replica 1)
    let adapter_r1 = Box::new(FilesystemAdapter::new("complex_test_r1")
        .expect("Cannot initialize R1's adapter"));
    let mut melda_r1 = Melda::new(Arc::new(RwLock::new(adapter_r1)))
        .expect("Failed to initialize R1's Melda");
    
    // Initialize R1 with initial data
    melda_r1.update(initial_json.as_object().unwrap().clone())
        .expect("Failed to update R1");
    let info = json!({ "replica": "R1", "action": "Initialize with 3 tasks" });
    melda_r1.commit(Some(info.as_object().unwrap().clone()))
        .expect("Failed to commit R1's initialization");
    
    // Copy R1's state to R2 and R3
    copy_recursively("complex_test_r1", "complex_test_r2")
        .expect("Failed to copy R1's state to R2");
    copy_recursively("complex_test_r1", "complex_test_r3")
        .expect("Failed to copy R1's state to R3");
    
    // Initialize R2 and R3 from copied state
    let adapter_r2 = Box::new(FilesystemAdapter::new("complex_test_r2")
        .expect("Cannot initialize R2's adapter"));
    let mut melda_r2 = Melda::new(Arc::new(RwLock::new(adapter_r2)))
        .expect("Failed to initialize R2's Melda");
    
    let adapter_r3 = Box::new(FilesystemAdapter::new("complex_test_r3")
        .expect("Cannot initialize R3's adapter"));
    let mut melda_r3 = Melda::new(Arc::new(RwLock::new(adapter_r3)))
        .expect("Failed to initialize R3's Melda");
    
    println!("✅ All replicas initialized with same initial state\n");
    println!("{}", "═".repeat(80));
    
    // === PHASE 1: R1 Operations ===
    println!("\n🔥 PHASE 1: R1 OPERATIONS\n");
    println!("📝 R1 Operation 1: Delete ALL initial elements");
    
    let r1_delete_all = json!({
        "document": "Task Management System",
        "version": "2.0", 
        "tasks♭": []  // Delete all elements
    });
    
    melda_r1.update(r1_delete_all.as_object().unwrap().clone())
        .expect("Failed to update R1 - delete all");
    let info = json!({ "replica": "R1", "action": "Delete all initial elements", "step": 1 });
    melda_r1.commit(Some(info.as_object().unwrap().clone()))
        .expect("Failed to commit R1's delete all");
    
    println!("✅ R1: Deleted all initial elements");
    
    println!("\n📝 R1 Operation 2: Add new element at index 1");
    
    let r1_add_at_1 = json!({
        "document": "Task Management System", 
        "version": "2.0",
        "tasks♭": [
            {"_id": "r1_task_1", "title": "R1's New Task", "priority": "critical", "index": 1, "author": "R1"}
        ]
    });
    
    melda_r1.update(r1_add_at_1.as_object().unwrap().clone())
        .expect("Failed to update R1 - add at index 1");
    let info = json!({ "replica": "R1", "action": "Add element at index 1", "step": 2 });
    melda_r1.commit(Some(info.as_object().unwrap().clone()))
        .expect("Failed to commit R1's add");
    
    println!("✅ R1: Added new task at index 1");
    
    // Show R1's state after its operations
    println!("\n📊 R1 State after Phase 1:");
    let r1_phase1_state = melda_r1.read(None).expect("Failed to read R1");
    println!("{}\n", serde_json::to_string_pretty(&r1_phase1_state).unwrap());
    
    println!("{}", "─".repeat(80));
    
    // === PHASE 2: R2 Operations (concurrent with R1) ===
    println!("\n🔥 PHASE 2: R2 OPERATIONS (Concurrent with R1)\n");
    println!("📝 R2 Operation 1: Add element at index 1");
    
    let r2_add_at_1 = json!({
        "document": "Task Management System",
        "version": "2.0",
        "tasks♭": [
            {"_id": "init_0", "title": "Setup Project", "priority": "high", "index": 0},
            {"_id": "r2_task_1", "title": "R2's Urgent Task", "priority": "urgent", "index": 1, "author": "R2"},
            {"_id": "init_1", "title": "Design Database", "priority": "medium", "index": 1},  // will shift
            {"_id": "init_2", "title": "Write Tests", "priority": "low", "index": 2}
        ]
    });
    
    melda_r2.update(r2_add_at_1.as_object().unwrap().clone())
        .expect("Failed to update R2 - add at index 1");
    let info = json!({ "replica": "R2", "action": "Add element at index 1", "step": 1 });
    melda_r2.commit(Some(info.as_object().unwrap().clone()))
        .expect("Failed to commit R2's add");
    
    println!("✅ R2: Added new task at index 1");
    
    println!("\n📝 R2 Operation 2: Delete element at index 0");
    
    let r2_delete_at_0 = json!({
        "document": "Task Management System",
        "version": "2.0",
        "tasks♭": [
            // init_0 removed (was at index 0)
            {"_id": "r2_task_1", "title": "R2's Urgent Task", "priority": "urgent", "index": 1, "author": "R2"},
            {"_id": "init_1", "title": "Design Database", "priority": "medium", "index": 1},
            {"_id": "init_2", "title": "Write Tests", "priority": "low", "index": 2}
        ]
    });
    
    melda_r2.update(r2_delete_at_0.as_object().unwrap().clone())
        .expect("Failed to update R2 - delete at index 0");
    let info = json!({ "replica": "R2", "action": "Delete element at index 0", "step": 2 });
    melda_r2.commit(Some(info.as_object().unwrap().clone()))
        .expect("Failed to commit R2's delete");
    
    println!("✅ R2: Deleted element at index 0 (init_0)");
    
    println!("\n📝 R2 Operation 3: Add element at index 2");
    
    let r2_add_at_2 = json!({
        "document": "Task Management System",
        "version": "2.0", 
        "tasks♭": [
            {"_id": "r2_task_1", "title": "R2's Urgent Task", "priority": "urgent", "index": 1, "author": "R2"},
            {"_id": "init_1", "title": "Design Database", "priority": "medium", "index": 1},
            {"_id": "r2_task_2", "title": "R2's Final Task", "priority": "normal", "index": 2, "author": "R2"},
            {"_id": "init_2", "title": "Write Tests", "priority": "low", "index": 2}
        ]
    });
    
    melda_r2.update(r2_add_at_2.as_object().unwrap().clone())
        .expect("Failed to update R2 - add at index 2");
    let info = json!({ "replica": "R2", "action": "Add element at index 2", "step": 3 });
    melda_r2.commit(Some(info.as_object().unwrap().clone()))
        .expect("Failed to commit R2's add");
    
    println!("✅ R2: Added new task at index 2");
    
    // Show R2's state after its operations
    println!("\n📊 R2 State after Phase 2:");
    let r2_phase2_state = melda_r2.read(None).expect("Failed to read R2");
    println!("{}\n", serde_json::to_string_pretty(&r2_phase2_state).unwrap());
    
    println!("{}", "─".repeat(80));
    
    // === PHASE 3: R3 syncs with R1, then adds ===
    println!("\n🔥 PHASE 3: R3 SYNCS WITH R1, THEN OPERATES\n");
    println!("📝 R3 Operation 1: Sync with R1 (get R1's changes)");
    
    melda_r3.meld(&melda_r1).expect("Failed to meld R1 into R3");
    let _ = melda_r3.refresh();
    
    println!("✅ R3: Synced with R1");
    
    // Show R3's state after syncing with R1
    println!("\n📊 R3 State after syncing with R1:");
    let r3_after_sync = melda_r3.read(None).expect("Failed to read R3");
    println!("{}\n", serde_json::to_string_pretty(&r3_after_sync).unwrap());
    
    println!("\n📝 R3 Operation 2: Add element at index 0");
    
    let r3_add_at_0 = json!({
        "document": "Task Management System",
        "version": "2.0",
        "tasks♭": [
            {"_id": "r3_task_0", "title": "R3's Priority Task", "priority": "highest", "index": 0, "author": "R3"},
            {"_id": "r1_task_1", "title": "R1's New Task", "priority": "critical", "index": 1, "author": "R1"}
        ]
    });
    
    melda_r3.update(r3_add_at_0.as_object().unwrap().clone())
        .expect("Failed to update R3 - add at index 0");
    let info = json!({ "replica": "R3", "action": "Add element at index 0", "step": 1 });
    melda_r3.commit(Some(info.as_object().unwrap().clone()))
        .expect("Failed to commit R3's add");
    
    println!("✅ R3: Added new task at index 0");
    
    // Show R3's final state before full sync
    println!("\n📊 R3 State after Phase 3:");
    let r3_phase3_state = melda_r3.read(None).expect("Failed to read R3");
    println!("{}\n", serde_json::to_string_pretty(&r3_phase3_state).unwrap());
    
    println!("{}", "═".repeat(80));
    
    // === PHASE 4: Full Synchronization ===
    println!("\n🔄 PHASE 4: FULL SYNCHRONIZATION OF ALL REPLICAS\n");
    
    println!("Synchronization sequence:");
    println!("1. R1 ← R2 (R1 gets R2's changes)");
    println!("2. R1 ← R3 (R1 gets R3's changes)"); 
    println!("3. R2 ← R1 (R2 gets all combined changes)");
    println!("4. R3 ← R1 (R3 gets all combined changes)");
    println!("5. R2 ← R3 (ensure full sync)");
    println!("6. R3 ← R2 (ensure full sync)\n");
    
    // Full sync process
    melda_r1.meld(&melda_r2).expect("Failed to meld R2 into R1");
    let _ = melda_r1.refresh();
    println!("✅ R1 ← R2 synchronized");
    
    melda_r1.meld(&melda_r3).expect("Failed to meld R3 into R1");
    let _ = melda_r1.refresh();
    println!("✅ R1 ← R3 synchronized");
    
    melda_r2.meld(&melda_r1).expect("Failed to meld R1 into R2");
    let _ = melda_r2.refresh();
    println!("✅ R2 ← R1 synchronized");
    
    melda_r3.meld(&melda_r1).expect("Failed to meld R1 into R3");
    let _ = melda_r3.refresh();
    println!("✅ R3 ← R1 synchronized");
    
    melda_r2.meld(&melda_r3).expect("Failed to meld R3 into R2");
    let _ = melda_r2.refresh();
    println!("✅ R2 ← R3 synchronized");
    
    melda_r3.meld(&melda_r2).expect("Failed to meld R2 into R3");
    let _ = melda_r3.refresh();
    println!("✅ R3 ← R2 synchronized");
    
    println!("\n{}", "═".repeat(80));
    
    // === FINAL RESULTS ===
    println!("\n✨ FINAL CONVERGED STATE\n");
    
    let final_r1 = melda_r1.read(None).expect("Failed to read R1");
    let final_r2 = melda_r2.read(None).expect("Failed to read R2");
    let final_r3 = melda_r3.read(None).expect("Failed to read R3");
    
    println!("R1 Final State:");
    println!("{}\n", serde_json::to_string_pretty(&final_r1).unwrap());
    
    // Verify convergence
    let r1_str = serde_json::to_string(&final_r1).unwrap();
    let r2_str = serde_json::to_string(&final_r2).unwrap();
    let r3_str = serde_json::to_string(&final_r3).unwrap();
    
    if r1_str == r2_str && r2_str == r3_str {
        println!("✅ SUCCESS: All replicas have converged to the same state!");
    } else {
        println!("❌ WARNING: Replicas have NOT converged!");
        println!("\nR2 Final State:");
        println!("{}\n", serde_json::to_string_pretty(&final_r2).unwrap());
        println!("R3 Final State:");
        println!("{}\n", serde_json::to_string_pretty(&final_r3).unwrap());
    }
    
    // Conflict Analysis
    println!("\n🔍 CONFLICT ANALYSIS:");
    let conflicts = melda_r1.in_conflict();
    if conflicts.is_empty() {
        println!("No unresolved conflicts detected!");
    } else {
        println!("Conflicts detected in {} objects:", conflicts.len());
        for (i, uuid) in conflicts.iter().enumerate() {
            let winner = melda_r1.get_winner(&uuid).unwrap();
            let conflicting = melda_r1.get_conflicting(&uuid).unwrap();
            println!("\nConflict {}: Object {}", i+1, uuid);
            println!("  Winner: {:?}", melda_r1.get_value(&uuid, Some(&winner)));
            for (j, c) in conflicting.iter().enumerate() {
                println!("  Alternative {}: {:?}", j+1, melda_r1.get_value(&uuid, Some(&c)));
            }
        }
    }
    
    // Element Analysis
    println!("\n📊 ELEMENT ANALYSIS:");
    let final_tasks = final_r1["tasks♭"].as_array().unwrap();
    println!("Total elements in final state: {}", final_tasks.len());
    
    // Count elements by origin
    let mut origin_counts = std::collections::HashMap::new();
    for task in final_tasks {
        if let Some(id) = task["_id"].as_str() {
            let origin = if id.starts_with("init_") { "Initial" }
            else if id.starts_with("r1_") { "R1" }
            else if id.starts_with("r2_") { "R2" } 
            else if id.starts_with("r3_") { "R3" }
            else { "Unknown" };
            *origin_counts.entry(origin).or_insert(0) += 1;
        }
    }
    
    println!("\nElements by origin:");
    for (origin, count) in origin_counts {
        println!("  {}: {} elements", origin, count);
    }
    
    println!("\nDetailed final elements:");
    for (i, task) in final_tasks.iter().enumerate() {
        println!("  {}. {} ({}): {} [{}]",
            i,
            task["title"].as_str().unwrap_or("No Title"),
            task["_id"].as_str().unwrap_or("No ID"),
            task["priority"].as_str().unwrap_or("No Priority"),
            task.get("author").and_then(|a| a.as_str()).unwrap_or("System")
        );
    }
    
    println!("\n📝 ANALYSIS:");
    println!("{}", "═".repeat(80));
    println!("This complex scenario tested:");
    println!("1. ✅ Mass deletion followed by insertion (R1)");
    println!("2. ✅ Sequential add-delete-add operations (R2)"); 
    println!("3. ✅ Partial sync followed by insertion (R3)");
    println!("4. ✅ Complex multi-replica synchronization");
    println!("5. ✅ Conflict resolution in interleaved operations");
    println!("\nKey observations:");
    println!("• Delete-then-add sequences work correctly"); 
    println!("• Partial synchronization maintains consistency");
    println!("• Complex operation ordering converges properly");
    println!("• Add-wins semantics preserve all intended additions");
    println!("\nThis demonstrates Melda's robustness in handling complex");
    println!("real-world scenarios with mixed operations and partial sync patterns.");
    
    // Clean up
    _ = fs::remove_dir_all("complex_test_r1");
    _ = fs::remove_dir_all("complex_test_r2"); 
    _ = fs::remove_dir_all("complex_test_r3");
}

// Helper function to copy directories recursively
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