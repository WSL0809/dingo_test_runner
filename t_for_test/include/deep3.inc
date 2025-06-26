# Deep nesting level 3
--echo 执行 deep3.inc (级别 3)
CREATE TABLE IF NOT EXISTS deep_test (level INT);
INSERT INTO deep_test VALUES (3);
--echo deep3.inc 完成 