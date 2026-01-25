# Synthetic FCS File Generation - Analysis and Decision

## Current State

Multiple crates have their own test FCS file generation functions:

1. **`flow-fcs/src/tests.rs`**: Simple test data (5 events, basic channels)
2. **`flow-plots/src/tests.rs`**: Test data for plotting (100 events, linear sequences)
3. **`tru-ols/src/fcs_integration.rs`**: Test data for detector channels (5 events, FL1-A/FL2-A/FL3-A)
4. **`peacoqc-rs/examples/basic_usage.rs`**: Synthetic data generation (10k events, realistic distributions)
5. **`gates/tests/test_helpers.rs`**: Synthetic scatter data (configurable scenarios, realistic patterns)

## Dependency Analysis

### Current Dependencies

- **`flow-utils`**: No dependency on `flow-fcs` ✅
- **`flow-fcs`**: No dependency on `flow-utils` ✅
- **`flow-plots`**: Depends on `flow-fcs` ✅
- **`flow-gates`**: Depends on `flow-fcs` and `flow-utils` ✅

### Potential Cyclic Dependency Risk

If we move synthetic FCS generation to `flow-utils`:

```
flow-utils → flow-fcs (for Fcs struct)
flow-fcs → flow-utils? (currently no, but could in future)
```

**Risk Level**: Medium
- Currently no cycle exists
- `flow-fcs` might need `flow-utils` algorithms in the future
- Would create: `flow-utils` → `flow-fcs` → `flow-utils` (cycle)

## Recommendation

### Option 1: Keep Separate (Recommended) ✅

**Keep synthetic FCS generation in each crate's test helpers**

**Pros:**
- ✅ No dependency risk
- ✅ Each crate can customize for its needs
- ✅ Test helpers are crate-specific anyway
- ✅ No shared code to maintain

**Cons:**
- ⚠️ Some code duplication
- ⚠️ Different patterns across crates

**Implementation:**
- Keep `gates/tests/test_helpers.rs` as-is
- Document the pattern for other crates to follow
- Consider creating a shared example/template

### Option 2: Move to flow-utils (Not Recommended) ❌

**Move synthetic FCS generation to `flow-utils`**

**Pros:**
- ✅ Single source of truth
- ✅ Shared across all crates

**Cons:**
- ❌ Creates dependency: `flow-utils` → `flow-fcs`
- ❌ Risk of future cycle if `flow-fcs` needs `flow-utils`
- ❌ `flow-utils` should be algorithm-focused, not test-focused
- ❌ Test helpers are typically crate-specific

### Option 3: Create Separate Test Utilities Crate (Future Consideration)

**Create `flow-test-utils` crate**

**Pros:**
- ✅ No dependency cycles (test-only)
- ✅ Shared test utilities
- ✅ Can depend on all other crates

**Cons:**
- ⚠️ Additional crate to maintain
- ⚠️ Only used in tests/dev dependencies
- ⚠️ Overhead for simple test helpers

**When to Consider:**
- If test helpers become complex
- If many crates need identical functionality
- If we want to share test data files

## Decision

**Keep synthetic FCS generation in `gates/tests/test_helpers.rs`**

**Rationale:**
1. Test helpers are typically crate-specific
2. No dependency cycle risk
3. Each crate can customize for its needs
4. `flow-utils` should focus on algorithms, not test utilities
5. Code duplication is minimal and acceptable for test code

**Action Items:**
- [x] Document the pattern in this analysis
- [ ] Consider creating a template/example for other crates
- [ ] Document best practices for synthetic data generation

## Usage Pattern

For other crates that need synthetic FCS files:

```rust
// In crate/tests/test_helpers.rs or crate/tests/mod.rs
use flow_fcs::{Fcs, Header, Metadata, Parameter, TransformType, file::AccessWrapper, parameter::ParameterMap};
use polars::prelude::*;
use std::sync::Arc;

fn create_test_fcs() -> Result<Fcs, Box<dyn std::error::Error>> {
    // Create temp file
    let temp_path = std::env::temp_dir().join("test_fcs.tmp");
    std::fs::File::create(&temp_path)?.write_all(b"test")?;
    
    // Create DataFrame with test data
    let df = DataFrame::new(vec![
        Column::new("FSC-A".into(), vec![...]),
        // ... more columns
    ])?;
    
    // Create parameter map
    let mut params = ParameterMap::default();
    params.insert("FSC-A".into(), Parameter::new(&1, "FSC-A", "FSC-A", &TransformType::Linear));
    
    // Create Fcs struct
    Ok(Fcs {
        header: Header::new(),
        metadata: Metadata::new(),
        parameters: params,
        data_frame: Arc::new(df),
        file_access: AccessWrapper::new(temp_path.to_str().unwrap_or(""))?,
    })
}
```
