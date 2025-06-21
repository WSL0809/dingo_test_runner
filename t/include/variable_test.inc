# Variable test in sourced file
--echo 执行 variable_test.inc 开始

--echo 使用主文件中定义的变量: $table_name
CREATE TABLE $table_name (
    id INT PRIMARY KEY,
    name VARCHAR(50),
    value INT
);

--echo 在 include 文件中定义新变量
let $test_value = 100
let $test_name = 'sourced_data'

--echo 使用 include 文件中定义的变量插入数据
INSERT INTO $table_name VALUES (1, $test_name, $test_value);

--echo variable_test.inc 执行完成 