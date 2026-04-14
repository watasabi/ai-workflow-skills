//! TUI module for ai-workflow-skills.
//!
//! Provides a unified interactive terminal interface for skill management
//! using the ratatui framework.

pub mod app;
pub mod state;
pub mod widgets;

pub use app::run;
