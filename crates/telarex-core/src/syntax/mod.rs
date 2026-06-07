//! Syntax highlighting — stylesheet definitions and Tree-sitter-based highlighting.
//!
//! [`StyleSheet`] defines color palettes and token styles for theming.
//! [`TreeHighlighter`] uses language-specific Tree-sitter queries to apply
//! syntax highlighting to visible text ranges.

pub mod stylesheet;
pub mod tree_highlighter;
pub use stylesheet::StyleSheet;
pub use tree_highlighter::TreeHighlighter;
