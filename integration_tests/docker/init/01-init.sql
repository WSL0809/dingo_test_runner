-- 初始化测试数据库
-- 这个脚本会在MySQL容器启动时自动执行

CREATE DATABASE IF NOT EXISTS test_db;
USE test_db;

-- 创建测试用表
CREATE TABLE IF NOT EXISTS test_users (
    id INT AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(50) NOT NULL,
    email VARCHAR(100),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 插入测试数据
INSERT INTO test_users (name, email) VALUES 
    ('Alice', 'alice@example.com'),
    ('Bob', 'bob@example.com'),
    ('Charlie', 'charlie@example.com');

-- 创建数字测试表
CREATE TABLE IF NOT EXISTS test_numbers (
    id INT AUTO_INCREMENT PRIMARY KEY,
    value INT NOT NULL
);

-- 插入测试数据（用于排序测试）
INSERT INTO test_numbers (value) VALUES (3), (1), (4), (1), (5), (9), (2), (6);

-- 创建字符串测试表
CREATE TABLE IF NOT EXISTS test_strings (
    id INT AUTO_INCREMENT PRIMARY KEY,
    content TEXT
);

-- 插入测试数据（用于正则表达式测试）
INSERT INTO test_strings (content) VALUES 
    ('test123'),
    ('abc456def'),
    ('hello world'),
    ('pattern789matching');

-- 创建时间测试表
CREATE TABLE IF NOT EXISTS test_dates (
    id INT AUTO_INCREMENT PRIMARY KEY,
    date_str VARCHAR(20),
    parsed_date DATE
);

-- 权限配置
GRANT ALL PRIVILEGES ON test_db.* TO 'test_user'@'%';
FLUSH PRIVILEGES;