--echo 🎉 欢迎使用 MySQL Test Runner (Rust)

# 创建示例表
CREATE TABLE demo_table (
    id INTEGER PRIMARY KEY,
    message TEXT
);

# 插入数据
INSERT INTO demo_table (message) VALUES ('Hello, Rust!');
INSERT INTO demo_table (message) VALUES ('SQLite 调试很简单');

# 查询数据
SELECT * FROM demo_table;

--echo ✅ 测试完成！ 