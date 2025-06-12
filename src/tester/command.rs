//! Defines the Command structure for .test file commands.
//! This is used by the new, extensible command handler system.

#[derive(Debug, Clone)]
pub struct Command {
    pub name: String,
    pub args: String,
    pub line: usize,
} 