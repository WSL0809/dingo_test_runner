# Common setup script for source testing
--echo 执行 setup.inc 开始

CREATE TABLE IF NOT EXISTS common_table (
    id INT PRIMARY KEY,
    data VARCHAR(100),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

TRUNCATE TABLE common_table;

INSERT INTO common_table (id, data) VALUES 
(1, 'Common Data 1'),
(2, 'Common Data 2'),
(3, 'Common Data 3');

--echo setup.inc 执行完成 