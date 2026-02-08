# Mutation Testing Guide

## Overview

Arthropod uses [cargo-mutants](https://mutants.rs/) to verify test quality. Mutation testing introduces small code changes (mutants) and verifies that tests catch them.

**Goal**: Maintain >80% mutation score (% of mutants caught by tests).

## Quick Start

### Linux/macOS

```bash
# Install cargo-mutants
cargo install cargo-mutants

# Run on specific crate
cargo mutants -p flux-state --no-shuffle --timeout 60

# Run on all crates (slow - ~2+ hours)
cargo mutants --no-shuffle --timeout 300
```

### Windows (Known Issues)

⚠️ **cargo-mutants has a known bug on Windows** (as of v26.2.0):
```
Error: Failed to copy C:\Users\...\arthropod\nul to ...
Caused by: The parameter is incorrect. (os error 87)
```

**Workarounds:**
1. **Use WSL** (Windows Subsystem for Linux) - recommended
2. **Use CI** - mutation testing runs automatically in GitHub Actions (Linux)
3. **Track upstream**: https://github.com/sourcefrog/cargo-mutants/issues

### WSL Setup

```bash
# From WSL terminal
cd /mnt/c/Users/markm/arthropod
cargo install cargo-mutants
cargo mutants -p flux-state --no-shuffle --timeout 60
```

## Interpreting Results

### Mutation Score

```
Mutants tested: 61
Caught: 52 (85.2%)  ✅
Missed: 7 (11.5%)   ⚠️
Timeout: 2 (3.3%)
```

- **Caught**: Tests detected the mutation (good!)
- **Missed**: Mutation survived - test gap (needs attention)
- **Timeout**: Test took too long (may need exclusion)

### What to Do About Missed Mutants

**Example missed mutant:**
```rust
// Original:
fn calculate_total(a: i32, b: i32) -> i32 {
    a + b  // Mutated to: a - b
}
```

If tests pass with `a - b`, it means no test verifies addition works correctly.

**Fix:**
```rust
#[test]
fn test_calculate_total_adds_correctly() {
    assert_eq!(calculate_total(2, 3), 5);  // Would fail with subtraction
}
```

## Configuration

See `mutants.toml` in the repository root:

```toml
# Exclude paths from mutation testing
exclude_globs = [
    "examples/**",
    "benches/**",
    "crates/arthropod-test/**",
]

# Exclude specific functions
exclude_re = [
    "^test_.*",      # Test helpers
    ".*::drop$",     # Drop implementations
    ".*::default$",  # Default implementations
]

# Timeout settings
minimum_test_timeout = 60
timeout_multiplier = 5.0
```

## CI Integration

Mutation testing runs automatically:
- **Weekly**: Full workspace scan
- **On PR**: Scans only changed files
- **Threshold**: >80% mutation score required

### Check Status

View mutation testing results:
- GitHub Actions → Mutation Testing workflow
- Artifacts → Download `mutants-results`

## Best Practices

### 1. Write Tests That Verify Behavior

❌ **Bad** (will miss mutants):
```rust
#[test]
fn test_parse_works() {
    let result = parse("123");
    assert!(result.is_ok());  // Doesn't check the value!
}
```

✅ **Good** (catches mutants):
```rust
#[test]
fn test_parse_returns_correct_number() {
    let result = parse("123");
    assert_eq!(result.unwrap(), 123);  // Verifies the actual value
}
```

### 2. Test Edge Cases

Mutants often survive in edge case handling:

```rust
fn divide(a: i32, b: i32) -> Option<i32> {
    if b == 0 {  // Mutant: b != 0
        return None;
    }
    Some(a / b)
}

#[test]
fn test_divide_by_zero_returns_none() {
    assert_eq!(divide(10, 0), None);  // Catches the mutant!
}
```

### 3. Use Property-Based Testing

For complex logic, use [proptest](https://docs.rs/proptest):

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_sort_is_idempotent(mut vec: Vec<i32>) {
        vec.sort();
        let sorted = vec.clone();
        vec.sort();
        assert_eq!(vec, sorted);  // Catches sorting bugs
    }
}
```

### 4. Exclude Trivial Code

Some code isn't worth mutating:

```rust
// Exclude boilerplate
impl Default for Config {
    #[mutants::skip]  // Skip this - trivial delegation
    fn default() -> Self {
        Config::new()
    }
}
```

## Crate-Specific Notes

### flux-state (61 mutants expected)

Core reactive runtime - high test coverage critical.

**Focus areas:**
- Signal get/set logic
- Dependency tracking
- Effect cleanup
- Computed memoization

### arthropod-ecs (~80 mutants expected)

ECS integration layer.

**Focus areas:**
- System scheduling
- Component queries
- Resource management
- Scene synchronization

### widget-core (~120 mutants expected)

Widget implementation layer.

**Focus areas:**
- Layout calculations
- Event handling
- Reactive updates
- Validation logic

## Troubleshooting

### Mutation Testing Takes Too Long

```bash
# Test just one file
cargo mutants --file src/signal.rs

# Parallel execution (use available cores)
cargo mutants --jobs 0

# Shorter timeout
cargo mutants --timeout 30
```

### Too Many Timeouts

Increase timeout in `mutants.toml`:
```toml
minimum_test_timeout = 120  # 2 minutes
timeout_multiplier = 10.0
```

### False Positives

Some mutants can't be caught:

```rust
// This mutant might timeout legitimately
fn wait_ms(ms: u64) {
    thread::sleep(Duration::from_millis(ms));  // Mutated: ms + 1
}
```

**Solution**: Exclude with `#[mutants::skip]` and document why.

## Resources

- [cargo-mutants documentation](https://mutants.rs/)
- [Mutation testing explained](https://en.wikipedia.org/wiki/Mutation_testing)
- [Arthropod testing philosophy](../adr/0007-test-driven-development.md)

## Summary

**Mutation testing verifies test quality, not code quality.**

Good tests:
- ✅ Catch bugs (high mutation score)
- ✅ Verify behavior, not implementation
- ✅ Cover edge cases
- ✅ Are fast and reliable

Our target: **>80% mutation score** = tests actually catch bugs.
