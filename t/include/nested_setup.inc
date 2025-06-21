# Nested setup script - sources another file
--echo 执行 nested_setup.inc 开始

--echo 从 nested_setup.inc 中 source setup.inc
--source include/setup.inc

--echo 在 nested_setup.inc 中创建额外的表
CREATE TABLE IF NOT EXISTS source_test_table (
    id INT PRIMARY KEY,
    name VARCHAR(50)
);

INSERT INTO source_test_table VALUES (1, 'nested test');

--echo nested_setup.inc 执行完成 