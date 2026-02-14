# Primitive Completeness Matrix (UI-Kit Foundation Gate)

## Goal

Define the minimum primitive foundation Arthropod needs before moving from example-specific UI work to reusable UI-kit and template packaging.

## Packaging Gate

- `P0` items: must be complete before UI-kit packaging.
- `P1` items: should be complete for first public design-system release.
- `P2` items: can follow after first release.

---

## P0 (Must Have)

| Primitive Area | Current State | Missing Foundation | Acceptance Tests |
|---|---|---|---|
| Layout contract (flex + alignment) | Basic row/column sizing and gap exist. | `justify_content`, `align_items`, wrap, min/max constraints, predictable intrinsic sizing. | 1) `row_aligns_children_center_end` 2) `column_justify_space_between` 3) `flex_wrap_moves_overflow_items` 4) `min_max_constraints_are_respected` |
| Incremental layout + cache invalidation | Cache scaffolding exists. | Real cache keys, dependency tracking, and invalidation only for affected subtrees. | 1) `unchanged_subtree_layout_is_reused` 2) `style_change_invalidates_target_subtree_only` 3) benchmark: <10% frame regression at 1k nodes |
| Text input editing model | Basic cursor, insert/delete/backspace are present. | Selection model, clipboard cut/copy/paste, word navigation, home/end, multiline semantics, IME/composition hooks. | 1) `text_input_selection_replaces_range` 2) `clipboard_copy_paste_roundtrip` 3) `ctrl_backspace_deletes_word` 4) `home_end_moves_cursor_correctly` |
| Interaction/event routing | Click/key dispatch exists. | Pointer enter/leave/down/up/click state machine, focus transfer rules, capture/bubble semantics. | 1) `hover_enter_leave_fires_once_per_boundary_cross` 2) `pointer_capture_retains_drag_target` 3) `focus_moves_tab_order_and_skips_disabled` 4) `click_requires_down_and_up_on_same_target` |
| Theme/style resolution pipeline | Style and theme engines exist, widgets still style ad hoc in places. | Single style resolution path from tokens -> state variants -> widget render properties. | 1) `button_hover_uses_theme_variant_not_hardcoded_color` 2) `disabled_state_overrides_hover_state` 3) `global_theme_swap_updates_widget_tree` |
| Image paint parity in primitives | Image paint type exists; render path still incomplete in primitive pipeline. | Correct image fill sampling/path parity with solid/gradient fills. | 1) `image_fill_renders_without_fallback_color` 2) `image_fill_respects_corner_radius_clip` 3) `image_fill_handles_scale_modes` |
| Scene lifecycle safety | Node remove APIs exist. | Safe subtree removal, no orphaned descendants, deterministic cleanup hooks. | 1) `remove_parent_removes_all_descendants` 2) `removed_nodes_not_queryable` 3) `entity_scene_links_are_cleaned_on_remove` |

---

## P1 (Should Have)

| Primitive Area | Current State | Missing Foundation | Acceptance Tests |
|---|---|---|---|
| A11y semantic bridge | A11y model is strong at engine level. | Automatic widget -> role/state/action mapping; focus announcements and labels. | 1) `button_maps_to_accessible_button_role` 2) `text_input_exposes_value_and_label` 3) `disabled_controls_are_non_actionable` |
| Typography primitives in widget API | Engine text supports advanced fields. | Widget-level API for weight, style, line-height, letter spacing, alignment, fallback families. | 1) `text_widget_applies_weight_and_line_height` 2) `text_alignment_reflects_layout_direction` |
| Scroll and viewport primitives | Basic list/grid primitives exist. | Scroll container, viewport clipping, wheel/trackpad behavior, virtualization contract. | 1) `scroll_container_clips_children` 2) `mouse_wheel_updates_scroll_offset` 3) `virtualized_list_renders_visible_range_only` |
| Overlay/portal primitives | Stack exists. | Modal, popover, tooltip, portal/layer root with deterministic z-order and focus trap. | 1) `modal_traps_focus_until_closed` 2) `popover_anchors_and_repositions_on_resize` |
| Form behavior primitives | Form container and field widgets exist. | Validation lifecycle, dirty/touched metadata, submit gating, async submit states. | 1) `required_field_blocks_submit` 2) `async_submit_disables_controls_until_complete` |

---

## P2 (Can Follow)

| Primitive Area | Missing Foundation | Acceptance Tests |
|---|---|---|
| Motion primitives | Tokenized duration/easing + standard transitions (hover/focus/enter/exit). | `transition_tokens_apply_consistently_across_widgets` |
| Advanced visual effects | Unified frosted/noise/effect presets with quality tiers. | `glass_preset_matches_expected_effect_stack` |
| Design-token export/import | Token pipeline interoperability (Figma/JSON/schema versioning). | `token_json_roundtrip_preserves_semantic_roles` |

---

## Suggested Execution Order

1. Layout contract + cache invalidation (`P0`)
2. Interaction routing + text input model (`P0`)
3. Theme/style resolution unification (`P0`)
4. Image paint parity + scene lifecycle safety (`P0`)
5. A11y bridge + typography exposure (`P1`)
6. Scroll/overlay/form behavior (`P1`)

## Release Readiness Criteria

- UI-kit packaging allowed only when all `P0` acceptance tests pass in CI.
- Design-system release candidate when `P0` is fully green and at least 80% of `P1` tests are green.
- Benchmark guardrails:
  - No >10% regression in layout/update/render benches for 1k widgets.
  - No correctness regressions in focus/input/a11y integration tests.
