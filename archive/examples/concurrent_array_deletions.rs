// Concurrent Array Deletions Example for Melda CRDT
// 
// This example demonstrates how Melda handles concurrent deletions of the same
// element from an array across multiple replicas. This tests the idempotence
// property of delete operations in CRDT systems.
//
// The simulation involves:
// - 3 replicas (Alice, Bob, and Charlie)
// - All replicas start with the same array containing multiple elements
// - All replicas delete the same element simultaneously
// - We then merge all replicas and observe idempotent behavior

use melda::{filesystemadapter::FilesystemAdapter, melda::Melda};
use serde_json::json;
use std::fs;
use std::io;
use std::path::Path;
use std::sync::{Arc, RwLock};

fn main() {
    // Clean up any existing test directories
    _ = fs::remove_dir_all("delete_test_alice");
    _ = fs::remove_dir_all("delete_test_bob");
    _ = fs::remove_dir_all("delete_test_charlie");
    
    println!("=== Melda CRDT: Concurrent Array Deletions Test ===\n");
    println!("This example tests how Melda handles concurrent deletions");
    println!("of the same element from an array across multiple replicas.\n");
    println!("Testing: IDEMPOTENCE of delete operations\n");
    
    // Initial JSON document with a populated flattened array
    let initial_json = json!({
        "document": "Shared Shopping List",
        "version": "1.0",
        "items♭": [
            {"_id": "item_1", "name": "Apples", "quantity": 5, "category": "fruit"},
            {"_id": "item_2", "name": "Bread", "quantity": 2, "category": "bakery"},
            {"_id": "item_3", "name": "Milk", "quantity": 1, "category": "dairy"},
            {"_id": "item_4", "name": "Cheese", "quantity": 3, "category": "dairy"},
            {"_id": "item_5", "name": "Bananas", "quantity": 6, "category": "fruit"}
        ]
    });
    
    println!("📄 INITIAL JSON STATE:");
    println!("{}\n", serde_json::to_string_pretty(&initial_json).unwrap());
    println!("Target for deletion: item_3 (Milk) - all replicas will delete this simultaneously\n");
    println!("{}", "─".repeat(60));
    
    // Create three replicas using filesystem adapters
    println!("\n🔄 CREATING THREE REPLICAS\n");
    
    // Alice's replica
    let adapter_alice = Box::new(FilesystemAdapter::new("delete_test_alice")
        .expect("Cannot initialize Alice's adapter"));
    let mut melda_alice = Melda::new(Arc::new(RwLock::new(adapter_alice)))
        .expect("Failed to initialize Alice's Melda");
    
    // Initialize all replicas with the same initial state
    println!("📝 Initializing all replicas with the same initial state...");
    
    // Alice initializes
    melda_alice.update(initial_json.as_object().unwrap().clone())
        .expect("Failed to update Alice");
    let info = json!({ "author": "Alice", "action": "Initialize shared shopping list" });
    melda_alice.commit(Some(info.as_object().unwrap().clone()))
        .expect("Failed to commit Alice's initialization");
    
    // Share Alice's state with Bob and Charlie by copying the filesystem
    copy_recursively("delete_test_alice", "delete_test_bob")
        .expect("Failed to copy Alice's state to Bob");
    copy_recursively("delete_test_alice", "delete_test_charlie")
        .expect("Failed to copy Alice's state to Charlie");
    
    // Bob's replica  
    let adapter_bob = Box::new(FilesystemAdapter::new("delete_test_bob")
        .expect("Cannot initialize Bob's adapter"));
    let mut melda_bob = Melda::new(Arc::new(RwLock::new(adapter_bob)))
        .expect("Failed to initialize Bob's Melda");
    
    // Charlie's replica
    let adapter_charlie = Box::new(FilesystemAdapter::new("delete_test_charlie")
        .expect("Cannot initialize Charlie's adapter"));
    let mut melda_charlie = Melda::new(Arc::new(RwLock::new(adapter_charlie)))
        .expect("Failed to initialize Charlie's Melda");
    
    println!("✅ All replicas initialized and synchronized\n");
    println!("{}", "─".repeat(60));
    
    // Now each replica deletes the same item (item_3 - Milk) concurrently
    println!("\n🗑️  CONCURRENT DELETIONS OF SAME ITEM\n");
    
    // Alice deletes item_3 (Milk)
    println!("👩 ALICE's deletion:");
    let alice_delete = json!({
        "document": "Shared Shopping List",
        "version": "1.0",
        "items♭": [
            {"_id": "item_1", "name": "Apples", "quantity": 5, "category": "fruit"},
            {"_id": "item_2", "name": "Bread", "quantity": 2, "category": "bakery"},
            // item_3 (Milk) removed
            {"_id": "item_4", "name": "Cheese", "quantity": 3, "category": "dairy"},
            {"_id": "item_5", "name": "Bananas", "quantity": 6, "category": "fruit"}
        ]
    });
    println!("Delta: Removing item_3 (Milk) from the shopping list");
    melda_alice.update(alice_delete.as_object().unwrap().clone())
        .expect("Failed to update Alice");
    let info = json!({ "author": "Alice", "action": "Delete item_3 (Milk)" });
    melda_alice.commit(Some(info.as_object().unwrap().clone()))
        .expect("Failed to commit Alice's delete");
    println!("✅ Alice committed the deletion\n");
    
    // Bob also deletes item_3 (Milk) - same operation
    println!("👨 BOB's deletion:");
    let bob_delete = json!({
        "document": "Shared Shopping List",
        "version": "1.0",
        "items♭": [
            {"_id": "item_1", "name": "Apples", "quantity": 5, "category": "fruit"},
            {"_id": "item_2", "name": "Bread", "quantity": 2, "category": "bakery"},
            // item_3 (Milk) removed
            {"_id": "item_4", "name": "Cheese", "quantity": 3, "category": "dairy"},
            {"_id": "item_5", "name": "Bananas", "quantity": 6, "category": "fruit"}
        ]
    });
    println!("Delta: Removing item_3 (Milk) from the shopping list (SAME as Alice)");
    melda_bob.update(bob_delete.as_object().unwrap().clone())
        .expect("Failed to update Bob");
    let info = json!({ "author": "Bob", "action": "Delete item_3 (Milk)" });
    melda_bob.commit(Some(info.as_object().unwrap().clone()))
        .expect("Failed to commit Bob's delete");
    println!("✅ Bob committed the deletion\n");
    
    // Charlie also deletes item_3 (Milk) - same operation
    println!("👦 CHARLIE's deletion:");
    let charlie_delete = json!({
        "document": "Shared Shopping List",
        "version": "1.0",
        "items♭": [
            {"_id": "item_1", "name": "Apples", "quantity": 5, "category": "fruit"},
            {"_id": "item_2", "name": "Bread", "quantity": 2, "category": "bakery"},
            // item_3 (Milk) removed
            {"_id": "item_4", "name": "Cheese", "quantity": 3, "category": "dairy"},
            {"_id": "item_5", "name": "Bananas", "quantity": 6, "category": "fruit"}
        ]
    });
    println!("Delta: Removing item_3 (Milk) from the shopping list (SAME as Alice and Bob)");
    melda_charlie.update(charlie_delete.as_object().unwrap().clone())
        .expect("Failed to update Charlie");
    let info = json!({ "author": "Charlie", "action": "Delete item_3 (Milk)" });
    melda_charlie.commit(Some(info.as_object().unwrap().clone()))
        .expect("Failed to commit Charlie's delete");
    println!("✅ Charlie committed the deletion\n");
    
    println!("{}", "─".repeat(60));
    
    // Show each replica's local state before merging
    println!("\n📊 LOCAL STATES AFTER DELETIONS (Before Merging):\n");
    
    println!("Alice's view:");
    let alice_state = melda_alice.read(None).expect("Failed to read Alice");
    println!("{}\n", serde_json::to_string_pretty(&alice_state).unwrap());
    
    println!("Bob's view:");
    let bob_state = melda_bob.read(None).expect("Failed to read Bob");
    println!("{}\n", serde_json::to_string_pretty(&bob_state).unwrap());
    
    println!("Charlie's view:");
    let charlie_state = melda_charlie.read(None).expect("Failed to read Charlie");
    println!("{}\n", serde_json::to_string_pretty(&charlie_state).unwrap());
    
    println!("Note: All replicas should show the same state (item_3 deleted) since they performed identical operations.");
    println!("{}", "─".repeat(60));
    
    // Now merge all replicas to test idempotence
    println!("\n🔄 MERGING ALL REPLICAS\n");
    println!("Merge order and operations:");
    println!("1. Alice merges with Bob (identical delete operations)");
    println!("2. Alice merges with Charlie (identical delete operations)");
    println!("3. Bob merges with Alice (all identical delete operations)");
    println!("4. Charlie merges with Alice (all identical delete operations)\n");
    
    // Perform the merges
    melda_alice.meld(&melda_bob).expect("Failed to meld Bob into Alice");
    let _ = melda_alice.refresh();
    println!("✅ Alice merged with Bob");
    
    melda_alice.meld(&melda_charlie).expect("Failed to meld Charlie into Alice");
    let _ = melda_alice.refresh();
    println!("✅ Alice merged with Charlie");
    
    melda_bob.meld(&melda_alice).expect("Failed to meld Alice into Bob");
    let _ = melda_bob.refresh();
    println!("✅ Bob merged with Alice (getting all updates)");
    
    melda_charlie.meld(&melda_alice).expect("Failed to meld Alice into Charlie");
    let _ = melda_charlie.refresh();
    println!("✅ Charlie merged with Alice (getting all updates)\n");
    
    println!("{}", "─".repeat(60));
    
    // Show final converged state
    println!("\n✨ FINAL CONVERGED STATE:\n");
    
    let final_alice = melda_alice.read(None).expect("Failed to read Alice");
    let final_bob = melda_bob.read(None).expect("Failed to read Bob");
    let final_charlie = melda_charlie.read(None).expect("Failed to read Charlie");
    
    println!("Alice's final view:");
    println!("{}\n", serde_json::to_string_pretty(&final_alice).unwrap());
    
    // Verify convergence
    let alice_str = serde_json::to_string(&final_alice).unwrap();
    let bob_str = serde_json::to_string(&final_bob).unwrap();
    let charlie_str = serde_json::to_string(&final_charlie).unwrap();
    
    if alice_str == bob_str && bob_str == charlie_str {
        println!("✅ SUCCESS: All replicas have converged to the same state!");
        println!("✅ IDEMPOTENCE VERIFIED: Multiple identical delete operations resulted in the same final state!");
    } else {
        println!("❌ WARNING: Replicas have NOT converged!");
        println!("\nBob's final view:");
        println!("{}\n", serde_json::to_string_pretty(&final_bob).unwrap());
        println!("Charlie's final view:");
        println!("{}\n", serde_json::to_string_pretty(&final_charlie).unwrap());
    }
    
    // Check for conflicts
    println!("\n🔍 CONFLICT ANALYSIS:");
    
    let conflicts = melda_alice.in_conflict();
    if conflicts.is_empty() {
        println!("No conflicts detected - Identical delete operations are properly idempotent!");
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
    
    // Verify the specific item was deleted
    println!("\n🔍 DELETION VERIFICATION:");
    let final_items = final_alice["items♭"].as_array().unwrap();
    let item_3_exists = final_items.iter().any(|item| {
        item["_id"].as_str() == Some("item_3")
    });
    
    if !item_3_exists {
        println!("✅ CONFIRMED: item_3 (Milk) has been successfully deleted from all replicas");
        println!("✅ ITEMS REMAINING: {} out of original 5", final_items.len());
        
        println!("\nRemaining items:");
        for item in final_items {
            println!("  - {} ({}): {} units", 
                item["name"].as_str().unwrap_or("Unknown"),
                item["_id"].as_str().unwrap_or("No ID"),
                item["quantity"].as_i64().unwrap_or(0)
            );
        }
    } else {
        println!("❌ ERROR: item_3 (Milk) still exists in the final state!");
    }
    
    println!("\n📝 ANALYSIS:");
    println!("{}", "─".repeat(60));
    println!("When multiple replicas delete the same element concurrently:");
    println!("1. Each replica performs the identical delete operation independently");
    println!("2. The delete operations are idempotent - applying them multiple times has the same effect");
    println!("3. After merging, the element remains deleted (not 'un-deleted' or duplicated)");
    println!("4. All replicas converge to the same state with the element properly removed");
    println!("5. No conflicts arise from identical operations on the same element");
    println!("\nThis demonstrates the correctness of Melda's delete operation idempotence,");
    println!("which is essential for distributed systems where the same operation might");
    println!("be performed by multiple nodes simultaneously.");
    
    // Clean up test directories
    _ = fs::remove_dir_all("delete_test_alice");
    _ = fs::remove_dir_all("delete_test_bob");
    _ = fs::remove_dir_all("delete_test_charlie");
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