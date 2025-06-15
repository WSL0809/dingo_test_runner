//! The representation of a query.
//! A .test file is a collection of queries.

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

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Query {
    pub query_type: QueryType,
    pub query: String,
    pub line: usize,
}
