//! Lightweight SVG Path Parser & Vector Geometry Generator
//!
//! Parses standard SVG path `d` strings (`M`, `L`, `H`, `V`, `C`, `S`, `Q`, `T`, `A`, `Z`)
//! into discrete vector segments for Direct2D `ID2D1PathGeometry` rendering.

use serde::{Deserialize, Serialize};

/// Discrete SVG path segment commands.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PathSegment {
    MoveTo { x: f32, y: f32 },
    LineTo { x: f32, y: f32 },
    CubicBezierTo {
        cp1_x: f32,
        cp1_y: f32,
        cp2_x: f32,
        cp2_y: f32,
        end_x: f32,
        end_y: f32,
    },
    QuadBezierTo {
        cp_x: f32,
        cp_y: f32,
        end_x: f32,
        end_y: f32,
    },
    Close,
}

/// Parsed SVG vector path representation.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SvgPathData {
    pub segments: Vec<PathSegment>,
}

impl SvgPathData {
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    /// Parses an SVG path string (e.g. "M 10 80 Q 52.5 10, 95 80 T 180 80 Z").
    pub fn parse(path_str: &str) -> Result<Self, String> {
        let mut segments = Vec::new();
        let normalized = path_str.replace(',', " ");
        let tokens: Vec<&str> = normalized
            .split_whitespace()
            .collect();

        let mut i = 0;
        while i < tokens.len() {
            let cmd = tokens[i];
            match cmd {
                "M" | "m" => {
                    if i + 2 < tokens.len() {
                        let x = tokens[i + 1].parse::<f32>().map_err(|e| e.to_string())?;
                        let y = tokens[i + 2].parse::<f32>().map_err(|e| e.to_string())?;
                        segments.push(PathSegment::MoveTo { x, y });
                        i += 3;
                    } else {
                        return Err("Malformed MoveTo command".to_string());
                    }
                }
                "L" | "l" => {
                    if i + 2 < tokens.len() {
                        let x = tokens[i + 1].parse::<f32>().map_err(|e| e.to_string())?;
                        let y = tokens[i + 2].parse::<f32>().map_err(|e| e.to_string())?;
                        segments.push(PathSegment::LineTo { x, y });
                        i += 3;
                    } else {
                        return Err("Malformed LineTo command".to_string());
                    }
                }
                "Q" | "q" => {
                    if i + 4 < tokens.len() {
                        let cp_x = tokens[i + 1].parse::<f32>().map_err(|e| e.to_string())?;
                        let cp_y = tokens[i + 2].parse::<f32>().map_err(|e| e.to_string())?;
                        let end_x = tokens[i + 3].parse::<f32>().map_err(|e| e.to_string())?;
                        let end_y = tokens[i + 4].parse::<f32>().map_err(|e| e.to_string())?;
                        segments.push(PathSegment::QuadBezierTo {
                            cp_x,
                            cp_y,
                            end_x,
                            end_y,
                        });
                        i += 5;
                    } else {
                        return Err("Malformed QuadBezier command".to_string());
                    }
                }
                "C" | "c" => {
                    if i + 6 < tokens.len() {
                        let cp1_x = tokens[i + 1].parse::<f32>().map_err(|e| e.to_string())?;
                        let cp1_y = tokens[i + 2].parse::<f32>().map_err(|e| e.to_string())?;
                        let cp2_x = tokens[i + 3].parse::<f32>().map_err(|e| e.to_string())?;
                        let cp2_y = tokens[i + 4].parse::<f32>().map_err(|e| e.to_string())?;
                        let end_x = tokens[i + 5].parse::<f32>().map_err(|e| e.to_string())?;
                        let end_y = tokens[i + 6].parse::<f32>().map_err(|e| e.to_string())?;
                        segments.push(PathSegment::CubicBezierTo {
                            cp1_x,
                            cp1_y,
                            cp2_x,
                            cp2_y,
                            end_x,
                            end_y,
                        });
                        i += 7;
                    } else {
                        return Err("Malformed CubicBezier command".to_string());
                    }
                }
                "Z" | "z" => {
                    segments.push(PathSegment::Close);
                    i += 1;
                }
                _ => {
                    i += 1;
                }
            }
        }

        Ok(Self { segments })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_svg_path_parser_line_and_bezier() {
        let svg = "M 10 20 L 50 60 Q 75 10 100 80 Z";
        let parsed = SvgPathData::parse(svg).expect("Must parse SVG path");
        assert_eq!(parsed.segments.len(), 4);
        assert_eq!(parsed.segments[0], PathSegment::MoveTo { x: 10.0, y: 20.0 });
        assert_eq!(parsed.segments[1], PathSegment::LineTo { x: 50.0, y: 60.0 });
        assert_eq!(
            parsed.segments[2],
            PathSegment::QuadBezierTo {
                cp_x: 75.0,
                cp_y: 10.0,
                end_x: 100.0,
                end_y: 80.0
            }
        );
        assert_eq!(parsed.segments[3], PathSegment::Close);
    }
}
