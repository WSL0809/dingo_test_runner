# Common cleanup script for source testing
--echo 执行 cleanup.inc 开始

DROP TABLE IF EXISTS common_table;
DROP TABLE IF EXISTS source_test_table;

--echo cleanup.inc 执行完成 