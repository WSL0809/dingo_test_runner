//! The representation of a query.
//! A .test file is a collection of queries.

use regex::Regex;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum QueryType {
    Query,
    Exec,
    Admin,
    Error,
    Fatal,
    Echo,
    Sleep,
    ReplaceRegex,
    ReplaceColumn,
    Replace,
    Let,
    Eval,
    Require,
    Source,
    Comment,
    Unknown,
    Connect,
    Connection,
    Disconnect,
    Delimiter,
    DisableQueryLog,
    EnableQueryLog,
    DisableResultLog,
    EnableResultLog,
    SortedResult,
    EnableSortResult,
    DisableSortResult,
    ChangeUser,
    EndOfFile,
    BeginConcurrent,
    EndConcurrent,
    Concurrent,
    VerticalResults,
    HorizontalResults,
    Send,
    Recv,
    Wait,
    RealSleep,
    QueryAsync,
    Block,
    Unblock,
    Checkpoint,
    Restart,
    Ping,
    Skip,
    Exit,
    // Control flow commands
    If,
    While,
    End,
    CloseBrace, // } for closing control flow blocks
}

/// 一次性修饰符、期望错误等元数据
#[derive(Debug, Clone, Default)]
pub struct QueryOptions {
    /// 该 SQL 预期抛出的错误码/错误名集合；空表示不期望错误
    pub expected_errors: Vec<String>,
    /// --replace_regex 收集的替换规则（顺序保持）
    pub replace_regex: Vec<(Regex, String)>,
    /// 是否对结果进行排序 (--sorted_result)
    pub sorted_result: bool,
}

#[derive(Debug, Clone)]
pub struct Query {
    pub query_type: QueryType,
    pub query: String,
    pub line: usize,
    pub options: QueryOptions,
}
