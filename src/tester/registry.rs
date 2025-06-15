//! The command registry for mapping command names to handlers.

use super::command::Command;
use super::handlers;
use super::tester::Tester;
use anyhow::Result;
use once_cell::sync::Lazy;
use std::collections::HashMap;

pub type CommandExecutor = fn(&mut Tester, &Command) -> Result<()>;

pub static COMMAND_REGISTRY: Lazy<HashMap<&'static str, CommandExecutor>> = Lazy::new(|| {
    let mut m: HashMap<&'static str, CommandExecutor> = HashMap::new();

    // Register handlers here.
    m.insert("sleep", handlers::sleep::execute);
    m.insert("echo", handlers::echo::execute);
    m.insert("exec", handlers::exec::execute);
    m.insert("connect", handlers::connect::execute);
    m.insert("connection", handlers::connection::execute);
    m.insert("disconnect", handlers::disconnect::execute);
    
    // Log control commands
    m.insert("disable_query_log", handlers::query_log::disable_query_log);
    m.insert("enable_query_log", handlers::query_log::enable_query_log);
    m.insert("disable_result_log", handlers::result_log::disable_result_log);
    m.insert("enable_result_log", handlers::result_log::enable_result_log);
    
    // Result modifiers
    m.insert("sorted_result", handlers::sorted_result::execute);
    m.insert("replace_regex", handlers::replace_regex::execute);
    m.insert("error", handlers::error::execute);
    
    // Variable commands
    m.insert("let", handlers::let_handler::execute);
    
    m
}); 