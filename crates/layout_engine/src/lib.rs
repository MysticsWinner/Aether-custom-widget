pub mod position_store;
pub use position_store::{WidgetPositionSpec, WidgetPositionStore};

use taffy::prelude::*;
use widget_parser::WidgetManifest;

pub struct ComputedBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

pub struct WidgetLayoutSolver {
    taffy: TaffyTree<()>,
}

impl WidgetLayoutSolver {
    pub fn new() -> Self {
        Self {
            taffy: TaffyTree::new(),
        }
    }

    /// Solves flexbox layout coordinates for a widget manifest given container DPI scale
    pub fn solve_layout(&mut self, manifest: &WidgetManifest, scale_factor: f32) -> Result<ComputedBounds, String> {
        let scaled_width = manifest.layout.width * scale_factor;
        let scaled_height = manifest.layout.height * scale_factor;

        let style = Style {
            size: Size {
                width: Dimension::Length(scaled_width),
                height: Dimension::Length(scaled_height),
            },
            flex_direction: FlexDirection::Column,
            ..Default::default()
        };

        let root_node = self.taffy.new_leaf(style).map_err(|e| e.to_string())?;

        self.taffy
            .compute_layout(
                root_node,
                Size {
                    width: AvailableSpace::Definite(scaled_width),
                    height: AvailableSpace::Definite(scaled_height),
                },
            )
            .map_err(|e| e.to_string())?;

        let layout = self.taffy.layout(root_node).map_err(|e| e.to_string())?;

        Ok(ComputedBounds {
            x: layout.location.x,
            y: layout.location.y,
            width: layout.size.width,
            height: layout.size.height,
        })
    }
}

impl Default for WidgetLayoutSolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solve_layout() {
        let manifest_str = r#"
            [metadata]
            id = "test_widget"
            name = "Test Widget"
            author = "Test Author"
            version = "1.0.0"
            update_interval_ms = 1000

            [layout]
            width = 300.0
            height = 150.0

            [[elements]]
            id = "elem1"
            element_type = "text"
        "#;

        let manifest: WidgetManifest = toml::from_str(manifest_str).unwrap();
        let mut solver = WidgetLayoutSolver::new();
        let bounds = solver.solve_layout(&manifest, 1.0).unwrap();

        assert_eq!(bounds.width, 300.0);
        assert_eq!(bounds.height, 150.0);
    }
}
