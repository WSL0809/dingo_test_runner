pub mod connect;
pub mod connection;
pub mod disconnect;
pub mod echo;
pub mod error;
pub mod eval;
pub mod exec;
pub mod let_handler;
pub mod query_log;
pub mod replace_regex;
pub mod result_log;
pub mod sleep;
pub mod sorted_result;

// New enhanced syntax handlers
pub mod var_operations;
pub mod batch_operations;
pub mod transaction_operations;
