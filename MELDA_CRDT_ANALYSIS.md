# Melda CRDT: Delete-Insert Duplication Issue

## Critical Limitation: Move Operations Create Duplicates

### The Problem
Melda cannot safely implement concurrent move operations due to its separate handling of delete and insert operations.

### Example Scenario
```
Initial state: [item_A, item_B, item_C]

User 1: Move item_B to position 0
  1. Delete item_B
  2. Insert item_B at position 0

User 2: Move item_B to position 2  
  1. Delete item_B
  2. Insert item_B at position 2

Expected result: [item_B, item_A, item_C] OR [item_A, item_C, item_B]
Actual result:   [item_B, item_A, item_C, item_B]  // DUPLICATES!
```

### Root Cause
- **Idempotent deletes**: Both users delete item_B, but it's only removed once
- **Add-wins inserts**: Both users insert item_B, both insertions are preserved
- **Result**: One deletion + two insertions = duplication

### Impact
- Drag-and-drop interfaces create duplicates
- List reordering operations are unsafe
- Cannot implement move operations without coordination
- Any delete+reinsert pattern in concurrent scenarios causes duplication

### Workarounds
1. Use unique IDs for each move operation
2. Implement application-level deduplication
3. Add coordination/locking for move operations
4. Avoid move operations entirely

This is a fundamental limitation of CRDTs that treat delete and insert as separate operations rather than providing atomic move primitives.