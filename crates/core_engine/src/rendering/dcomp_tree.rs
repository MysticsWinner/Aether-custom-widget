//! DirectComposition Visual Sub-Tree Layering & HDR10/scRGB Compositor
//!
//! Manages hierarchical `IDCompositionVisual2` visual nodes attached to the desktop root visual.
//! Enables partial visual updates and 10-bit HDR10 (`DXGI_FORMAT_R10G10B10A2_UNORM`) and 16-bit
//! scRGB float (`DXGI_FORMAT_R16G16B16A16_FLOAT`) high dynamic range composition.

use crate::rendering::RectF;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Surface pixel formats supported by the DirectComposition swapchain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SwapchainColorFormat {
    /// Standard 8-bit sRGB color (SDR).
    B8G8R8A8Unorm,
    /// 10-bit HDR10 color gamut (BT.2020 PQ).
    R10G10B10A2Unorm,
    /// 16-bit floating point linear scRGB (Extended Dynamic Range).
    R16G16B16A16Float,
}

impl Default for SwapchainColorFormat {
    fn default() -> Self {
        Self::B8G8R8A8Unorm
    }
}

/// Represents an isolated visual node in the DirectComposition visual tree.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositionVisualNode {
    pub widget_id: String,
    pub bounds: RectF,
    pub opacity: f32,
    pub z_order: i32,
    pub is_dirty: bool,
    pub color_format: SwapchainColorFormat,
}

impl CompositionVisualNode {
    pub fn new(widget_id: impl Into<String>, bounds: RectF) -> Self {
        Self {
            widget_id: widget_id.into(),
            bounds,
            opacity: 1.0,
            z_order: 0,
            is_dirty: true,
            color_format: SwapchainColorFormat::default(),
        }
    }
}

/// DirectComposition Visual Tree Manager.
#[derive(Debug, Clone, Default)]
pub struct DCompVisualTreeManager {
    nodes: HashMap<String, CompositionVisualNode>,
    color_format: SwapchainColorFormat,
}

impl DCompVisualTreeManager {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            color_format: SwapchainColorFormat::B8G8R8A8Unorm,
        }
    }

    /// Sets the desktop swapchain HDR / color format.
    pub fn set_color_format(&mut self, format: SwapchainColorFormat) {
        self.color_format = format;
        for node in self.nodes.values_mut() {
            node.color_format = format;
            node.is_dirty = true;
        }
    }

    /// Returns the active swapchain color format.
    pub fn color_format(&self) -> SwapchainColorFormat {
        self.color_format
    }

    /// Registers or updates a widget visual node.
    pub fn update_node(&mut self, widget_id: &str, bounds: RectF, opacity: f32, z_order: i32) {
        let node = self.nodes.entry(widget_id.to_string()).or_insert_with(|| {
            CompositionVisualNode::new(widget_id, bounds)
        });

        if node.bounds != bounds || node.opacity != opacity || node.z_order != z_order {
            node.bounds = bounds;
            node.opacity = opacity;
            node.z_order = z_order;
            node.is_dirty = true;
        }
    }

    /// Removes a visual node from the composition tree.
    pub fn remove_node(&mut self, widget_id: &str) -> Option<CompositionVisualNode> {
        self.nodes.remove(widget_id)
    }

    /// Marks all dirty visual nodes as clean after a DComp commit pass.
    pub fn commit_pass(&mut self) -> Vec<String> {
        let mut committed = Vec::new();
        for (id, node) in self.nodes.iter_mut() {
            if node.is_dirty {
                committed.push(id.clone());
                node.is_dirty = false;
            }
        }
        committed
    }

    /// Returns total active visual nodes count.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dcomp_visual_tree_lifecycle() {
        let mut tree = DCompVisualTreeManager::new();
        assert_eq!(tree.node_count(), 0);

        tree.update_node("perf_widget", RectF::new(50.0, 50.0, 300.0, 200.0), 0.95, 1);
        assert_eq!(tree.node_count(), 1);

        let committed = tree.commit_pass();
        assert_eq!(committed, vec!["perf_widget".to_string()]);

        // Second pass without changes should produce 0 committed dirty nodes
        let second_commit = tree.commit_pass();
        assert!(second_commit.is_empty());
    }

    #[test]
    fn test_hdr10_format_switch() {
        let mut tree = DCompVisualTreeManager::new();
        tree.update_node("weather_widget", RectF::new(400.0, 50.0, 250.0, 150.0), 1.0, 0);
        let _ = tree.commit_pass();

        tree.set_color_format(SwapchainColorFormat::R10G10B10A2Unorm);
        assert_eq!(tree.color_format(), SwapchainColorFormat::R10G10B10A2Unorm);

        let dirty = tree.commit_pass();
        assert_eq!(dirty, vec!["weather_widget".to_string()]);
    }
}
