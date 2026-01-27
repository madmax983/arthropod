//! Utilities for converting Arthropod concepts to Windows UIA

use crate::node::Role;
use windows::Win32::UI::Accessibility::*;

/// Map Arthropod Role to UIA Control Type ID
pub fn role_to_control_type(role: Role) -> UIA_CONTROLTYPE_ID {
    match role {
        Role::Button => UIA_ButtonControlTypeId,
        Role::Checkbox => UIA_CheckBoxControlTypeId,
        Role::Radio => UIA_RadioButtonControlTypeId,
        Role::Textbox => UIA_EditControlTypeId,
        Role::Slider => UIA_SliderControlTypeId,
        Role::ProgressBar => UIA_ProgressBarControlTypeId,
        Role::Group => UIA_GroupControlTypeId,
        Role::List => UIA_ListControlTypeId,
        Role::ListItem => UIA_ListItemControlTypeId,
        Role::Grid => UIA_DataGridControlTypeId,
        Role::GridCell => UIA_DataItemControlTypeId,
        Role::Heading { .. } => UIA_HeaderControlTypeId,
        Role::Paragraph => UIA_TextControlTypeId,
        Role::Region => UIA_GroupControlTypeId,
        Role::Main => UIA_GroupControlTypeId,
        Role::Navigation => UIA_GroupControlTypeId,
        Role::Search => UIA_GroupControlTypeId,
        Role::Form => UIA_GroupControlTypeId,
        Role::Alert => UIA_TextControlTypeId,
        Role::Dialog => UIA_WindowControlTypeId,
        Role::Tooltip => UIA_ToolTipControlTypeId,
    }
}
