// Concurrent Array Insertions Example for Melda CRDT
// 
// This example demonstrates how Melda handles concurrent insertions at the same
// position in an array across multiple replicas. This is a common challenge in
// CRDT systems as multiple users may try to insert elements at the same index
// simultaneously.
//
// The simulation involves:
// - 3 replicas (Alice, Bob, and Charlie)
// - Each replica starts with the same initial array
// - Each replica inserts an element at position 1 (index 1) concurrently
// - We then merge all replicas and observe the final convergent state

use melda::{filesystemadapter::FilesystemAdapter, melda::Melda};
use serde_json::json;
use std::fs;
use std::io;
use std::path::Path;
use std::sync::{Arc, RwLock};

fn main() {
    // Clean up any existing test directories
    _ = fs::remove_dir_all("array_test_alice");
    _ = fs::remove_dir_all("array_test_bob");
    _ = fs::remove_dir_all("array_test_charlie");
    
    println!("=== Melda CRDT: Concurrent Array Insertions Test ===\n");
    println!("This example tests how Melda handles concurrent insertions");
    println!("at the same position in an array across multiple replicas.\n");
    
    // Initial JSON document with a flattened array
    // The ♭ suffix tells Melda to track array elements individually
    let initial_json = json!({
        "document": "Shared Task List",
        "version": "1.0",
        "tasks♭": [
            {"_id": "task_0", "title": "Initial Task", "position": 0},
            {"_id": "task_2", "title": "Final Task", "position": 2}
        ]
    });
    
    println!("📄 INITIAL JSON STATE:");
    println!("{}\n", serde_json::to_string_pretty(&initial_json).unwrap());
    println!("Note: Array has positions 0 and 2, with position 1 empty.\n");
    println!("{}", "─".repeat(60));
    
    // Create three replicas using filesystem adapters
    println!("\n🔄 CREATING THREE REPLICAS\n");
    
    // Alice's replica
    let adapter_alice = Box::new(FilesystemAdapter::new("array_test_alice")
        .expect("Cannot initialize Alice's adapter"));
    let mut melda_alice = Melda::new(Arc::new(RwLock::new(adapter_alice)))
        .expect("Failed to initialize Alice's Melda");
    
    // Bob's replica  
    let adapter_bob = Box::new(FilesystemAdapter::new("array_test_bob")
        .expect("Cannot initialize Bob's adapter"));
    let mut melda_bob = Melda::new(Arc::new(RwLock::new(adapter_bob)))
        .expect("Failed to initialize Bob's Melda");
    
    // Charlie's replica
    let adapter_charlie = Box::new(FilesystemAdapter::new("array_test_charlie")
        .expect("Cannot initialize Charlie's adapter"));
    let mut melda_charlie = Melda::new(Arc::new(RwLock::new(adapter_charlie)))
        .expect("Failed to initialize Charlie's Melda");
    
    // Initialize all replicas with the same initial state
    println!("📝 Initializing all replicas with the same initial state...");
    
    // Alice initializes
    melda_alice.update(initial_json.as_object().unwrap().clone())
        .expect("Failed to update Alice");
    let info = json!({ "author": "Alice", "action": "Initialize shared document" });
    melda_alice.commit(Some(info.as_object().unwrap().clone()))
        .expect("Failed to commit Alice's initialization");
    
    // Share Alice's state with Bob and Charlie by copying the filesystem
    copy_recursively("array_test_alice", "array_test_bob")
        .expect("Failed to copy Alice's state to Bob");
    copy_recursively("array_test_alice", "array_test_charlie")
        .expect("Failed to copy Alice's state to Charlie");
    
    // Reload Bob and Charlie with the copied state
    let adapter_bob = Box::new(FilesystemAdapter::new("array_test_bob")
        .expect("Cannot initialize Bob's adapter"));
    let mut melda_bob = Melda::new(Arc::new(RwLock::new(adapter_bob)))
        .expect("Failed to initialize Bob's Melda");
    
    let adapter_charlie = Box::new(FilesystemAdapter::new("array_test_charlie")
        .expect("Cannot initialize Charlie's adapter"));
    let mut melda_charlie = Melda::new(Arc::new(RwLock::new(adapter_charlie)))
        .expect("Failed to initialize Charlie's Melda");
    
    println!("✅ All replicas initialized and synchronized\n");
    println!("{}", "─".repeat(60));
    
    // Now each replica makes a concurrent insertion at position 1
    println!("\n🚀 CONCURRENT INSERTIONS AT POSITION 1\n");
    
    // Alice inserts her task at position 1
    println!("👩 ALICE's insertion:");
    let alice_update = json!({
        "document": "Shared Task List",
        "version": "1.0",
        "tasks♭": [
            {"_id": "task_0", "title": "Initial Task", "position": 0},
            {"_id": "alice_task", "title": "Alice's Important Task", "position": 1, "author": "Alice"},
            {"_id": "task_2", "title": "Final Task", "position": 2}
        ]
    });
    println!("Delta: Adding task with _id: 'alice_task' at position 1");
    melda_alice.update(alice_update.as_object().unwrap().clone())
        .expect("Failed to update Alice");
    let info = json!({ "author": "Alice", "action": "Insert task at position 1" });
    melda_alice.commit(Some(info.as_object().unwrap().clone()))
        .expect("Failed to commit Alice's update");
    println!("✅ Alice committed her insertion\n");
    
    // Bob inserts his task at position 1
    println!("👨 BOB's insertion:");
    let bob_update = json!({
        "document": "Shared Task List",
        "version": "1.0",
        "tasks♭": [
            {"_id": "task_0", "title": "Initial Task", "position": 0},
            {"_id": "bob_task", "title": "Bob's Urgent Task", "position": 1, "author": "Bob"},
            {"_id": "task_2", "title": "Final Task", "position": 2}
        ]
    });
    println!("Delta: Adding task with _id: 'bob_task' at position 1");
    melda_bob.update(bob_update.as_object().unwrap().clone())
        .expect("Failed to update Bob");
    let info = json!({ "author": "Bob", "action": "Insert task at position 1" });
    melda_bob.commit(Some(info.as_object().unwrap().clone()))
        .expect("Failed to commit Bob's update");
    println!("✅ Bob committed his insertion\n");
    
    // Charlie inserts his task at position 1
    println!("👦 CHARLIE's insertion:");
    let charlie_update = json!({
        "document": "Shared Task List",
        "version": "1.0",
        "tasks♭": [
            {"_id": "task_0", "title": "Initial Task", "position": 0},
            {"_id": "charlie_task", "title": "Charlie's Critical Task", "position": 1, "author": "Charlie"},
            {"_id": "task_2", "title": "Final Task", "position": 2}
        ]
    });
    println!("Delta: Adding task with _id: 'charlie_task' at position 1");
    melda_charlie.update(charlie_update.as_object().unwrap().clone())
        .expect("Failed to update Charlie");
    let info = json!({ "author": "Charlie", "action": "Insert task at position 1" });
    melda_charlie.commit(Some(info.as_object().unwrap().clone()))
        .expect("Failed to commit Charlie's update");
    println!("✅ Charlie committed his insertion\n");
    
    println!("{}", "─".repeat(60));
    
    // Show each replica's local state before merging
    println!("\n📊 LOCAL STATES BEFORE MERGING:\n");
    
    println!("Alice's view:");
    let alice_state = melda_alice.read(None).expect("Failed to read Alice");
    println!("{}\n", serde_json::to_string_pretty(&alice_state).unwrap());
    
    println!("Bob's view:");
    let bob_state = melda_bob.read(None).expect("Failed to read Bob");
    println!("{}\n", serde_json::to_string_pretty(&bob_state).unwrap());
    
    println!("Charlie's view:");
    let charlie_state = melda_charlie.read(None).expect("Failed to read Charlie");
    println!("{}\n", serde_json::to_string_pretty(&charlie_state).unwrap());
    
    println!("{}", "─".repeat(60));
    
    // Now merge all replicas
    println!("\n🔄 MERGING ALL REPLICAS\n");
    println!("Merge order and delta application:");
    println!("1. Alice merges with Bob (receives Bob's delta)");
    println!("2. Alice merges with Charlie (receives Charlie's delta)");
    println!("3. Bob merges with Alice (receives Alice's and Charlie's deltas)");
    println!("4. Charlie merges with Alice (receives Alice's and Bob's deltas)\n");
    
    // Perform the merges
    melda_alice.meld(&melda_bob).expect("Failed to meld Bob into Alice");
    melda_alice.refresh();
    println!("✅ Alice merged with Bob");
    
    melda_alice.meld(&melda_charlie).expect("Failed to meld Charlie into Alice");
    melda_alice.refresh();
    println!("✅ Alice merged with Charlie");
    
    melda_bob.meld(&melda_alice).expect("Failed to meld Alice into Bob");
    melda_bob.refresh();
    println!("✅ Bob merged with Alice (getting all updates)");
    
    melda_charlie.meld(&melda_alice).expect("Failed to meld Alice into Charlie");
    melda_charlie.refresh();
    println!("✅ Charlie merged with Alice (getting all updates)\n");
    
    println!("{}", "─".repeat(60));
    
    // Show final converged state
    println!("\n✨ FINAL CONVERGED STATE:\n");
    
    let final_alice = melda_alice.read(None).expect("Failed to read Alice");
    let final_bob = melda_bob.read(None).expect("Failed to read Bob");
    let final_charlie = melda_charlie.read(None).expect("Failed to read Charlie");
    
    
    // Verify convergence
    let alice_str = serde_json::to_string(&final_alice).unwrap();
    let bob_str = serde_json::to_string(&final_bob).unwrap();
    let charlie_str = serde_json::to_string(&final_charlie).unwrap();
    
    if alice_str == bob_str && bob_str == charlie_str {
        println!("✅ SUCCESS: All replicas have converged to the same state!");
        println!("{}\n", serde_json::to_string_pretty(&final_alice).unwrap());

    } else {
        println!("❌ WARNING: Replicas have NOT converged!");
        println!("\nAlice's final view:");
        println!("{}\n", serde_json::to_string_pretty(&final_alice).unwrap());
        println!("\nBob's final view:");
        println!("{}\n", serde_json::to_string_pretty(&final_bob).unwrap());
        println!("Charlie's final view:");
        println!("{}\n", serde_json::to_string_pretty(&final_charlie).unwrap());
    }
    
    // Check for conflicts
    println!("\n🔍 CONFLICT ANALYSIS:");
    
    let conflicts = melda_alice.in_conflict();
    if conflicts.is_empty() {
        println!("No conflicts detected - Melda handled concurrent insertions successfully!");
    } else {
        println!("Conflicts detected in objects: {:?}", conflicts);
        for uuid in conflicts {
            let winner = melda_alice.get_winner(&uuid).unwrap();
            let conflicting = melda_alice.get_conflicting(&uuid).unwrap();
            println!("\nObject {}: ", uuid);
            println!("  Winner: {:?}", melda_alice.get_value(&uuid, Some(&winner)));
            for c in conflicting {
                println!("  Conflict: {:?}", melda_alice.get_value(&uuid, Some(&c)));
            }
        }
    }
    

    // Clean up test directories
    _ = fs::remove_dir_all("array_test_alice");
    _ = fs::remove_dir_all("array_test_bob");
    _ = fs::remove_dir_all("array_test_charlie");
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