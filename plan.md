1. **Refactor `Select::build` (`crates/material-ui/src/inputs/select.rs`)**
   - Extract the theme resolution into a helper method or struct.
   - Extract the logic for building the trigger node into `build_trigger`.
   - Extract the logic for building the dropdown into `build_dropdown`.
2. **Refactor `Autocomplete::build` (`crates/material-ui/src/inputs/autocomplete.rs`)**
   - Apply similar extractions as `Select::build`.
3. **Refactor `Slider::build` (`crates/material-ui/src/inputs/slider.rs`)**
   - Extract theme resolution.
   - Extract nodes building into separate helper methods.
4. **Refactor `BottomNavigation::build` (`crates/material-ui/src/navigation/bottom_navigation.rs`)**
   - Extract theme resolution.
   - Extract inner item building loop into a `build_item` method.
5. **Verify changes**
   - Run `cargo fmt`, `cargo clippy`, and `cargo test` to ensure no behavior changes and code style is respected.
6. Complete pre commit steps.
7. Submit the PR with the title `⚒️ Forge: Refactor monolithic Widget build methods in material-ui`.
