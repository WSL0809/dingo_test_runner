//! Defines the Command structure for .test file commands.
//! This is used by the new, extensible command handler system.

#[derive(Debug, Clone, Default)]
pub struct Command {
    pub name: String,
    pub args: String,
    pub line: usize,
}

impl Command {
    /// Create a new command with the given name and arguments
    pub fn new(name: &str, args: &str, line: usize) -> Self {
        Self {
            name: name.to_string(),
            args: args.to_string(),
            line,
        }
    }

    /// Check if this command is of the given type
    pub fn is_type(&self, cmd_type: &str) -> bool {
        self.name == cmd_type
    }

    /// Get the command name
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Get the command arguments
    pub fn get_args(&self) -> &str {
        &self.args
    }

    /// Get the line number where this command was found
    pub fn get_line(&self) -> usize {
        self.line
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_new() {
        let cmd = Command::new("echo", "Hello World", 42);
        assert_eq!(cmd.name, "echo");
        assert_eq!(cmd.args, "Hello World");
        assert_eq!(cmd.line, 42);
    }

    #[test]
    fn test_command_default() {
        let cmd = Command::default();
        assert_eq!(cmd.name, "");
        assert_eq!(cmd.args, "");
        assert_eq!(cmd.line, 0);
    }

    #[test]
    fn test_command_is_type() {
        let cmd = Command::new("sleep", "1.5", 10);
        assert!(cmd.is_type("sleep"));
        assert!(!cmd.is_type("echo"));
        assert!(!cmd.is_type("Sleep")); // Case sensitive
    }

    #[test]
    fn test_command_getters() {
        let cmd = Command::new("connect", "server1", 15);
        assert_eq!(cmd.get_name(), "connect");
        assert_eq!(cmd.get_args(), "server1");
        assert_eq!(cmd.get_line(), 15);
    }

    #[test]
    fn test_command_clone() {
        let cmd1 = Command::new("test", "args", 20);
        let cmd2 = cmd1.clone();
        
        assert_eq!(cmd1.name, cmd2.name);
        assert_eq!(cmd1.args, cmd2.args);
        assert_eq!(cmd1.line, cmd2.line);
    }

    #[test]
    fn test_command_empty_args() {
        let cmd = Command::new("disconnect", "", 5);
        assert_eq!(cmd.get_args(), "");
        assert!(cmd.is_type("disconnect"));
    }

    #[test]
    fn test_command_multiline_args() {
        let args = "SELECT * FROM table\nWHERE id = 1";
        let cmd = Command::new("query", args, 30);
        assert_eq!(cmd.get_args(), args);
        assert!(cmd.get_args().contains('\n'));
    }
}
