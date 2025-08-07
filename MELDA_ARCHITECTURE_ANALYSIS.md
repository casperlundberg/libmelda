# Melda CRDT Architecture Analysis

## Executive Summary

This report presents findings from comprehensive testing of the libmelda CRDT implementation, revealing that **Melda is fundamentally a state-based CRDT despite its "Delta-State" naming**. The "delta" refers to storage optimization (.delta files), not operational semantics.

## Key Findings

### 1. State-Based Operations Only

**Finding**: Melda requires complete state in every `update()` operation.

**Evidence**: The `delta_simulation_test.rs` demonstrated that providing partial arrays causes deletion of omitted elements:
- Initial state: `[item_1, item_2, item_3]`
- Alice's "delta" update: `[item_4]` only
- Result: `[item_4]` - original items deleted
- Bob's "delta" update: `[item_5]` only  
- Result: `[item_5]` - original items deleted

### 2. No Incremental API

**Finding**: Melda provides no higher-level object manipulation methods.

**API Limitations**:
- Only `update(complete_object)` available
- No `insert()`, `append()`, `delete()`, or similar methods
- No SDK for managing arrays or nested objects
- Requires reading full state before any modification

### 3. Merge Algorithm Requirements

**Finding**: Melda's pivot-based merge algorithm requires complete arrays.

**Technical Details** (from `src/utils.rs:115-155`):
- `merge_arrays()` uses sophisticated pivot detection
- Needs full context to resolve concurrent modifications
- Cannot operate on partial/incremental updates
- Explains why state-based approach is mandatory

### 4. Developer Impact

**Challenges for Delta Operations**:
1. Must read full state before any modification
2. Must reconstruct complete object for every change  
3. Cannot perform simple append/insert operations
4. Risk of accidental data loss if elements are missed
5. High bandwidth usage for large documents

## Test Results Summary

### Delta Simulation Test

**Purpose**: Test what happens when simulating delta operations by providing only new elements.

**Scenario**:
- Start: `[item_1, item_2, item_3]`
- Alice adds: `[item_4]` only
- Bob adds: `[item_5]` only

**Results**:
- Alice's state: `[item_4]` (original items lost)
- Bob's state: `[item_5]` (original items lost)
- Final merged: `[item_4, item_5]` (all original items permanently lost)

**Conclusion**: Confirmed that Melda interprets partial arrays as complete state replacement.

## Architecture Implications

### Storage Optimization vs Operational Model

- **Storage**: Uses `.delta` and `.pack` files for efficiency
- **Operations**: Requires full state updates despite delta storage
- **Naming**: "Delta-State CRDT" refers to storage, not operation model

### Comparison with True Delta CRDTs

| Aspect | True Delta CRDT | Melda |
|--------|----------------|-------|
| Operations | Incremental updates | Complete state replacement |
| API | `insert()`, `delete()`, etc. | Only `update(full_state)` |
| Bandwidth | Minimal (deltas only) | High (complete state) |
| Complexity | Simple operations | Read-modify-write pattern |

## Recommendations

### For Developers

1. **Understand the Model**: Treat Melda as state-based, not delta-based
2. **Read-Modify-Write Pattern**: Always read current state before updates
3. **Careful State Management**: Ensure all elements are included in updates
4. **Consider Alternatives**: For true incremental operations, consider other CRDT libraries

### For the Melda Project

1. **Documentation Clarity**: Update naming and docs to reflect state-based nature
2. **API Enhancement**: Consider adding higher-level manipulation methods
3. **Developer Experience**: Provide utilities to simplify read-modify-write patterns

## Technical References

### Key Source Files
- `src/melda.rs` - Main API (only `update()` method)
- `src/utils.rs:115-155` - Pivot-based array merging algorithm
- `examples/simple.rs:99-108` - Official example showing full state replacement

### Test Evidence
- `examples/delta_simulation_test.rs` - Definitive proof of state-based behavior

## Conclusion

Melda is a well-engineered state-based CRDT with delta storage optimization. However, its operational model requires complete state updates, making true incremental delta operations impossible. Developers should understand this fundamental architectural constraint when designing applications with Melda.

The "Delta-State" naming is misleading from an operational perspective - Melda is state-based in practice, with delta optimization being purely a storage concern.