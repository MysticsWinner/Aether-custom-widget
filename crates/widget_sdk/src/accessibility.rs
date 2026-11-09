//! Windows UI Automation (UIA / MSAA) Accessibility Tree Provider (Phase 26)
//!
//! Exposes semantic roles, labels, live region announcements, and keyboard tab-index
//! traversal for WorkerW-hosted desktop widgets so screen readers (Narrator / NVDA)
//! can inspect and interact with widget elements.

use crate::rendering::RectF;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Semantic UI Automation control type role for a widget element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessibleRole {
    WidgetCard,
    Gauge,
    Button,
    Text,
    Sparkline,
    Region,
    ProgressBar,
}

/// Politeness level for live screen-reader announcements (`AriaLive` / UIA `NotificationProcessing`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LivePoliteness {
    Off,
    Polite,
    Assertive,
}

/// A single accessible node in a widget's UIA / MSAA tree.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccessibleNode {
    pub element_id: String,
    pub role: AccessibleRole,
    pub label: String,
    pub value_text: Option<String>,
    pub bounds: RectF,
    pub tab_index: Option<i32>,
    pub focusable: bool,
    pub live_politeness: LivePoliteness,
}

impl AccessibleNode {
    pub fn new(
        element_id: impl Into<String>,
        role: AccessibleRole,
        label: impl Into<String>,
        bounds: RectF,
    ) -> Self {
        Self {
            element_id: element_id.into(),
            role,
            label: label.into(),
            value_text: None,
            bounds,
            tab_index: None,
            focusable: false,
            live_politeness: LivePoliteness::Off,
        }
    }

    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value_text = Some(value.into());
        self
    }

    pub fn with_tab_index(mut self, tab_index: i32) -> Self {
        self.tab_index = Some(tab_index);
        self.focusable = true;
        self
    }

    pub fn with_live_region(mut self, politeness: LivePoliteness) -> Self {
        self.live_politeness = politeness;
        self
    }

    /// Formats a screen-reader narration string combining role, label, and current value.
    pub fn narration_text(&self) -> String {
        match &self.value_text {
            Some(val) => format!("{}: {} ({:?})", self.label, val, self.role),
            None => format!("{} ({:?})", self.label, self.role),
        }
    }
}

/// Per-widget UI Automation tree managing focus traversal and screen reader announcements.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccessibilityTree {
    nodes: Vec<AccessibleNode>,
    focused_element_id: Option<String>,
    announcements: VecDeque<(LivePoliteness, String)>,
}

impl AccessibilityTree {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers an accessible element node in the tree.
    pub fn register_node(&mut self, node: AccessibleNode) {
        self.nodes.push(node);
    }

    /// Clears all nodes before rebuilding on a layout pass.
    pub fn clear_nodes(&mut self) {
        self.nodes.clear();
    }

    /// Returns all registered accessible nodes.
    pub fn nodes(&self) -> &[AccessibleNode] {
        &self.nodes
    }

    /// Advances keyboard focus to the next focusable node ordered by `tab_index` ascending.
    pub fn focus_next(&mut self) -> Option<&AccessibleNode> {
        let mut focusables: Vec<&AccessibleNode> = self
            .nodes
            .iter()
            .filter(|n| n.focusable && n.tab_index.is_some())
            .collect();
        focusables.sort_by_key(|n| n.tab_index.unwrap_or(i32::MAX));

        if focusables.is_empty() {
            self.focused_element_id = None;
            return None;
        }

        let next_idx = match &self.focused_element_id {
            Some(cur_id) => focusables
                .iter()
                .position(|n| &n.element_id == cur_id)
                .map(|idx| (idx + 1) % focusables.len())
                .unwrap_or(0),
            None => 0,
        };

        let next_id = focusables[next_idx].element_id.clone();
        self.focused_element_id = Some(next_id.clone());
        self.nodes.iter().find(|n| n.element_id == next_id)
    }

    /// Returns the currently focused accessible node, if any.
    pub fn focused_node(&self) -> Option<&AccessibleNode> {
        let id = self.focused_element_id.as_ref()?;
        self.nodes.iter().find(|n| &n.element_id == id)
    }

    /// Queues a live screen-reader announcement (bounded to 32 entries).
    pub fn announce(&mut self, politeness: LivePoliteness, message: impl Into<String>) {
        if politeness == LivePoliteness::Off {
            return;
        }
        if self.announcements.len() >= 32 {
            self.announcements.pop_front();
        }
        self.announcements.push_back((politeness, message.into()));
    }

    /// Pops the next queued screen-reader announcement.
    pub fn pop_announcement(&mut self) -> Option<(LivePoliteness, String)> {
        self.announcements.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accessibility_tree_tab_order_and_announcements() {
        let mut tree = AccessibilityTree::new();
        tree.register_node(
            AccessibleNode::new(
                "btn_settings",
                AccessibleRole::Button,
                "Open Widget Settings",
                RectF::new(260.0, 12.0, 24.0, 24.0),
            )
            .with_tab_index(2),
        );
        tree.register_node(
            AccessibleNode::new(
                "gauge_cpu",
                AccessibleRole::Gauge,
                "CPU Utilization",
                RectF::new(16.0, 40.0, 280.0, 24.0),
            )
            .with_value("24.5%")
            .with_tab_index(1)
            .with_live_region(LivePoliteness::Polite),
        );

        let first = tree.focus_next().unwrap();
        assert_eq!(first.element_id, "gauge_cpu");
        assert_eq!(first.narration_text(), "CPU Utilization: 24.5% (Gauge)");

        let second = tree.focus_next().unwrap();
        assert_eq!(second.element_id, "btn_settings");

        // Wrap around
        let wrapped = tree.focus_next().unwrap();
        assert_eq!(wrapped.element_id, "gauge_cpu");

        tree.announce(LivePoliteness::Assertive, "High CPU temperature alert: 89C");
        let (politeness, msg) = tree.pop_announcement().unwrap();
        assert_eq!(politeness, LivePoliteness::Assertive);
        assert_eq!(msg, "High CPU temperature alert: 89C");
    }
}

