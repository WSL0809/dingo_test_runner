MySQL 测试运行器 mysql-test-runner.pl 及其 .test 文件标记深度解析摘要MySQL 测试套件是确保 MySQL 服务器稳定性和可靠性的关键组成部分，而 mysql-test-runner.pl (MTR) 则是执行此套件的核心工具。本报告旨在深度剖析 mysql-test-runner.pl 的功能，并重点研究其测试用例文件（.test 文件）中使用的特定命令标记。这些标记，例如 --echo 用于输出信息，--error 用于预期错误处理，--let 进行变量赋值，if 和 while 实现流程控制，--sleep 用于暂停执行，--source 引入外部脚本，以及 --query_vertical 和 --query_horizontal 控制结果显示格式，构成了 mysqltest 语言的基础。理解并有效运用这些命令对于编写健壮、确定且易于维护的测试用例至关重要，从而保障 MySQL 在多种环境下的功能正确性。一、 引言MySQL 测试套件的重要性MySQL 作为一个功能丰富且广泛应用的开源关系数据库管理系统，其代码库的复杂性要求一个全面且严格的测试机制。MySQL 测试套件 (MySQL Test Suite) 的存在，对于保证数据库服务器在不同操作系统、硬件平台以及各种配置下的稳定性、可靠性、功能正确性和性能表现至关重要。它不仅帮助开发人员在开发过程中及时发现和修复缺陷，也为用户提供了对其所使用 MySQL 版本质量的信心。mysql-test-runner.pl (MTR) 概述mysql-test-runner.pl，通常简称为 MTR，是一个用 Perl 语言编写的脚本，它是 MySQL 测试套件的主要驱动程序 1。MTR 的核心职责是自动化测试用例的执行流程。这包括启动一个或多个 mysqld 服务器实例（可能带有特定的配置参数），然后调用 mysqltest 工具来逐个运行 .test 文件中定义的测试用例 2。在执行过程中，MTR 会捕获 mysqltest 的输出，并将其与预定义的期望结果文件（通常是 .result 文件）进行比较，以判断测试是否通过。MTR 还负责管理测试套件的组织，允许选择性执行单个测试、一组测试或整个套件，并能处理不同的测试配置和环境设置 1。研究范围与目的本报告的核心目标是深入分析 mysql-test-runner.pl 的工作机制，特别是对其测试用例文件——.test 文件——内部使用的各种命令标记（markers）进行详细研究。这些命令标记是 mysqltest 程序解析和执行的指令，它们控制着测试的流程、与服务器的交互、错误处理以及结果的生成与验证。具体而言，本报告将重点探讨以下常用命令的功能、语法和应用场景：--echo、--error、--disable_abort_on_error、--enable_abort_on_error、--let、if、while、--sleep、--source、--query_vertical 和 --query_horizontal。通过对这些命令的解析，旨在为 MySQL 的开发者和测试工程师提供一份关于如何有效利用 MTR 框架编写高质量测试用例的参考。二、 .test 文件：MySQL 测试用例的核心.test 文件是 MySQL 测试套件中定义具体测试逻辑的单元。它们包含了指示 mysqltest 程序如何与 MySQL 服务器交互、执行哪些操作以及如何验证结果的指令。.test 文件的基本结构与约定.test 文件本质上是纯文本文件，其内容由一系列 mysqltest 命令和标准的 SQL 语句组成 3。这些命令和 SQL 语句共同定义了一个完整的测试场景。mysqltest 命令本身是不区分大小写的 4。注释在 .test 文件中扮演着重要角色，用于提高可读性和解释测试意图。以单个井号 # 开头的行被视为注释，其内容不会被复制到结果文件中。而以双井号 ## 开头的注释，在特定结果格式（如 result_format 2）下，会被逐字复制到结果文件中，这有助于在结果中保留更详细的上下文信息 4。默认情况下，每个命令或 SQL 语句以分号 ;作为结束分隔符。然而，为了能够测试包含分号的复杂语句，例如存储过程定义或多语句块，mysqltest 提供了 delimiter 命令。通过 delimiter <新分隔符>，可以将命令分隔符临时更改为指定的字符串（如 //），从而允许在命令块内部自由使用分号，之后再通过 delimiter ; 恢复默认分隔符 4。这一机制对于编写涉及存储程序单元 (Stored Program Units) 的测试至关重要。这些基本结构和约定是编写和理解 .test 文件的基础。它们确保了测试脚本的清晰度和一致性，使得开发人员能够有效地定义和执行各种测试场景。变量替换机制为了增强测试脚本的灵活性和可配置性，mysqltest 支持变量替换机制。在 .test 文件中，许多命令的文件名参数和文本参数都可以包含变量。这些变量以美元符号 $ 开头，例如 $MYSQL_TMP_DIR 代表 MySQL 测试框架使用的临时目录路径 4。在 mysqltest 执行这些命令之前，它会扫描参数中的变量引用，并将其替换为对应变量的实际值。如果需要表示字面上的美元符号，可以使用反斜杠进行转义，即 \$ 4。这种变量替换能力使得测试用例不必硬编码特定的路径、文件名或配置值，而是可以根据测试环境动态生成。例如，测试数据文件的路径、日志文件的名称等都可以通过变量来指定，从而提高了测试用例的可移植性和在不同环境下的适应性。在定义测试场景中的核心角色.test 文件通过其内部的 mysqltest 命令和 SQL 语句的组合，精确地定义和模拟了各种复杂的测试场景。这些场景可以涵盖：
功能验证：测试特定的 SQL 功能、服务器特性或 API 行为是否符合预期。
边界条件测试：探索系统在极端或非标准输入下的表现。
错误处理测试：验证服务器在遇到无效操作或错误条件时，能否正确报告错误并保持稳定。
回归测试：确保已修复的缺陷不会再次出现，以及新的代码变更没有破坏现有功能。
.test 文件的设计理念，在某种程度上体现了“测试即文档”和“可执行的规范”的思想。每一个 .test 文件都包含了一系列清晰的操作步骤（SQL 语句和 mysqltest 命令）3。与之对应的 .result 文件则明确定义了这些操作步骤执行后所期望的输出结果 2。此外，诸如 --echo 这样的命令允许在测试结果中插入描述性的文本，进一步阐释测试的意图和上下文 4。这种结构使得 .test 文件不仅仅是可执行的测试代码，更成为其所测试功能的详细文字描述和行为规范。任何需要理解特定 MySQL 功能预期行为的人，都可以通过阅读相关的 .test 和 .result 文件来获得精确的信息。从另一个角度看，mysqltest 语言本身可以被视为一种为数据库测试这一特定领域量身打造的领域特定语言 (DSL)。4 中列举的大量命令，如用于连接管理的 connect 和 disconnect，用于错误处理的 --error 和 disable_abort_on_error，用于结果格式化的 query_vertical 和 replace_regex，以及用于复制场景控制的 sync_slave_with_master 等，都远超出了通用脚本语言所提供的基本功能。这些命令直接映射到数据库测试过程中常见的操作和验证需求。这种 DSL 的设计，使得测试用例的编写更为简洁，更贴近测试人员的意图，从而显著提高了测试开发的效率和测试脚本的可读性。三、 mysqltest核心命令详解（.test 文件标记）mysqltest 提供了一套丰富的命令集，用于在 .test 文件中控制测试流程、与服务器交互、处理输出和验证结果。下表汇总了一些关键的 mysqltest 命令，随后的章节将对用户查询中特别关注的命令进行详细解析。命令 (Command)核心语法示例简要功能描述--echo <text>--echo "Starting test section"将文本输出到结果文件。--error <errlist>--error ER_NO_SUCH_TABLE声明下一条语句预期发生的错误。let $<var> = <value>let $count = 10;为 mysqltest 变量赋值。if (<condition>) {... }if ($count > 5) { --echo "Count is > 5"; }条件执行块。while (<condition>) {... }while ($i < 3) { inc $i; }循环执行块。--sleep <seconds>sleep 1.5;暂停脚本执行指定的秒数。--source <filename>--source include/setup.inc执行指定文件中的命令。--query_vertical <SQL>query_vertical SELECT * FROM t1;以垂直格式执行并显示 SQL 结果。--query_horizontal <SQL>query_horizontal SELECT * FROM t1;以水平格式执行并显示 SQL 结果。disable_abort_on_errordisable_abort_on_error;发生意外错误时，测试不中止。enable_abort_on_errorenable_abort_on_error;恢复发生意外错误时中止测试的默认行为。eval <statement_with_vars>eval CREATE TABLE $table_name (id INT);替换变量后执行语句。exec <shell_command>--exec ls -l $MYSQL_TMP_DIR执行外部 shell 命令。vertical_resultsvertical_results;设置后续查询结果默认为垂直格式。horizontal_resultshorizontal_results;设置后续查询结果默认为水平格式。replace_regex /patt/repl/[i]replace_regex /Timestamp:.*/Timestamp: <TS>/对结果进行正则表达式替换。sorted_resultsorted_result; SELECT col FROM unsorted_table;对结果集排序后进行比较。connect (params)connect (conn1,localhost,root,,test);创建新的数据库连接。disconnect <conn_name>disconnect conn1;关闭指定的数据库连接。connection <conn_name>connection conn1;切换到指定的数据库连接。write_file <file> [term]write_file $MYSQL_TMP_DIR/data.txt EOF将后续行写入文件。cat_file <file>cat_file $MYSQL_TMP_DIR/data.txt;将文件内容输出到结果。remove_file <file>remove_file $MYSQL_TMP_DIR/data.txt;删除文件。输出与日志控制这些命令主要用于控制测试过程中哪些信息被记录到 .result 文件中，以及如何格式化这些信息。

--echo <文本>

目的与功能：此命令用于将指定的文本字符串直接输出到测试结果（.result 文件）中。它通常被用来在结果文件中添加注释、说明测试的特定阶段、标记重要的检查点，或者简单地提高结果文件的可读性 4。
语法格式：--echo 文本内容 或 echo 文本内容。命令后的所有字符，直到行尾或分号（取决于上下文），都将被视为要输出的文本 4。
实用范例：
Code snippet--echo #
--echo # Test Case: Verify behavior of INSERT IGNORE
--echo # Author: MySQL Test Team
--echo # Date: 2023-10-27
--echo #
--echo # Step 1: Setup initial table and data
CREATE TABLE t_echo (id INT PRIMARY KEY, val VARCHAR(10));
INSERT INTO t_echo VALUES (1, 'one');
--echo # Step 2: Attempt to insert a duplicate key with IGNORE
INSERT IGNORE INTO t_echo VALUES (1, 'another one');
--echo # Step 3: Verify table content remains unchanged for id=1
SELECT * FROM t_echo WHERE id=1;

在这个例子中，--echo 命令被用来清晰地标记测试用例的元信息和各个测试步骤，使得 .result 文件更易于理解。



disable_query_log / enable_query_log

目的与功能：这两个命令用于控制是否将 .test 文件中执行的 SQL 语句本身回显到测试结果文件中。默认情况下，查询日志是启用的 (enable_query_log 的状态)，即所有执行的 SQL 语句都会出现在 .result 文件中。使用 disable_query_log 可以暂时关闭此回显，直到遇到 enable_query_log 或测试结束 4。这在执行一些冗长的设置脚本或者不希望某些敏感查询（尽管在测试环境中这较少见）出现在结果中时非常有用。
语法格式：disable_query_log; 和 enable_query_log; 4。
实用范例：
Code snippet--echo # Query logging is enabled by default.
CREATE TABLE t_query_log (a INT);
--disable_query_log
--echo # The following CREATE and INSERT statements will NOT appear in the result file.
CREATE TABLE t_hidden_setup (id INT);
INSERT INTO t_hidden_setup VALUES (1), (2), (3);
--enable_query_log
--echo # Query logging is now re-enabled.
SELECT * FROM t_query_log; -- This SELECT statement will appear in the result.
DROP TABLE t_query_log, t_hidden_setup;





disable_result_log / enable_result_log

目的与功能：与查询日志类似，这对命令控制 SQL 语句执行后产生的结果（包括返回的数据行、影响的行数、警告信息或错误信息）是否被输出到测试结果文件中。默认情况下，结果日志是启用的 (enable_result_log 的状态) 4。在某些情况下，如果一个查询返回大量数据而这些数据对于验证测试点并非必需，或者只是一个中间步骤，可以使用 disable_result_log 来隐藏其输出，以保持 .result 文件的简洁。
语法格式：disable_result_log; 和 enable_result_log; 4。
实用范例：
Code snippetCREATE TABLE t_result_log (id INT, data VARCHAR(100));
INSERT INTO t_result_log VALUES (1, 'Visible Data');
--echo # The result of this SELECT will be shown.
SELECT * FROM t_result_log;

--disable_result_log
--echo # Populating a large helper table, its result/affected rows will be hidden.
--disable_query_log # Also hiding the query itself for brevity
INSERT INTO t_result_log SELECT seq, CONCAT('Data ', seq) FROM seq_1_to_1000;
--enable_query_log
--enable_result_log
--echo # Result logging is now re-enabled.
--echo # Verifying count from the helper table.
SELECT COUNT(*) FROM t_result_log;
DROP TABLE t_result_log;




错误处理与验证在测试过程中，验证系统能否正确处理错误情况与验证其正常功能同等重要。mysqltest 提供了一套机制来声明预期的错误并控制测试在发生错误时的行为。

--error <错误码列表>

目的与功能：此命令用于声明紧随其后的下一条 mysqltest 命令或 SQL 语句预期会产生一个或多个特定的错误。<错误码列表> 可以是逗号分隔的一个或多个值，这些值可以是：MySQL 服务器返回的数字错误代码（例如 1062 表示重复键错误），标准的 SQLSTATE 值（必须以大写字母 S 开头，例如 S23000 对应完整性约束违反），或者是 MySQL 内部定义的符号错误名称（例如 ER_DUP_ENTRY 或 ER_NO_SUCH_TABLE）4。如果实际发生的错误与 --error 命令中声明的任何一个预期错误相符，则测试认为此步骤通过，并继续执行。如果语句成功执行而没有产生任何错误，或者产生了列表中未包含的其他错误，那么 mysqltest 通常会中止整个测试用例的执行（除非 disable_abort_on_error 命令生效）4。
语法格式：--error <错误码1>[, <错误码2>...] 或 error <错误码1>[, <错误码2>...] 4。
实用范例：
Code snippetCREATE TABLE t_error (id INT PRIMARY KEY);
INSERT INTO t_error VALUES (1);

--echo # Expecting ER_DUP_ENTRY (MySQL error 1062, SQLSTATE 23000) for duplicate key
--error ER_DUP_ENTRY
INSERT INTO t_error VALUES (1); -- This should fail with a duplicate key error.

--echo # Expecting ER_NO_SUCH_TABLE (MySQL error 1146, SQLSTATE 42S02)
--error 1146
SELECT * FROM non_existent_table;

--echo # Expecting specific SQLSTATE
--error S42S02
DROP TABLE non_existent_table_again;

--echo # Expecting one of two possible errors (e.g., platform-dependent or version-dependent)
--error ER_BAD_TABLE_ERROR, ER_NO_SUCH_TABLE
-- Tämä on esimerkkikomento, joka saattaa epäonnistua eri tavoin
-- CHECK TABLE for a table that might be corrupted or non-existent.

DROP TABLE t_error;

错误码的数值和符号名称可以在 MySQL 的错误信息文档中找到，例如 16 和 17 中提及的错误。



--disable_abort_on_error / --enable_abort_on_error

目的与功能：默认情况下 (enable_abort_on_error 的状态)，如果 mysqltest 执行的 SQL 语句返回了一个未通过 --error 命令明确声明为预期的服务器错误，测试将会立即中止。disable_abort_on_error 命令可以改变这一行为，使得测试在遇到这类意外错误时不会中止，而是继续执行后续的命令 4。当 disable_abort_on_error 生效时，最后一条 SQL 语句返回的错误码（如果发生错误）会被存储在内置的 mysqltest 变量 $mysql_errno 中，该变量可以在后续的测试逻辑（例如 if 语句）中使用。enable_abort_on_error 命令则用于恢复默认的错误中止行为。
语法格式：disable_abort_on_error; 和 enable_abort_on_error; 4。
实用范例：
Code snippet--echo # Disabling abort on error for a block of cleanup statements
--disable_abort_on_error
DROP TABLE IF EXISTS temp_table1; -- Continues even if temp_table1 doesn't exist (would cause ER_BAD_TABLE_ERROR)
--echo # Error code from previous DROP (if any): $mysql_errno
DROP VIEW IF EXISTS temp_view1;   -- Continues even if temp_view1 doesn't exist
--echo # Error code from previous DROP (if any): $mysql_errno
--enable_abort_on_error
--echo # Abort on error behavior is now re-enabled.
--error ER_NO_SUCH_TABLE
SELECT * FROM temp_table1; -- This will now cause test to abort if temp_table1 was indeed dropped.

7 的讨论详细阐述了 disable_abort_on_error 的行为，特别是它与 $mysql_errno 变量以及 --error 命令的交互。

--error 和 disable_abort_on_error 共同提供了一套复杂而灵活的错误处理策略，允许测试用例在不同的执行阶段拥有不同级别的容错能力。首先，--error 命令用于精确断言某个操作会产生特定的、预期的错误 4。这是测试错误路径正确性的主要手段。其次，disable_abort_on_error 允许测试脚本在执行某些可能失败但失败并非关键的操作时（例如，尝试删除一个可能不存在的临时表作为清理步骤），能够继续执行下去，而不是因为这些非核心的失败而中断整个测试流程 7。在这种模式下，发生的任何 SQL 错误（无论是否预期）都不会中止测试，其错误码会被捕获到 $mysql_errno 变量中。
值得注意的是，正如 7 中所揭示的，即使在 disable_abort_on_error 模式下，如果一个语句前置了 --error <expected_error> 来声明一个预期的错误，但实际执行时发生了与 <expected_error> 不同 的错误，或者语句意外地 成功执行（即没有发生预期的错误），测试仍然会中止。这种情况分别被称为“错误不匹配 (wrong error)”或“预期错误但语句成功 (statement succeeded when error expected)”。
这种分层控制机制赋予了测试开发者极大的灵活性：

他们可以精确地测试特定错误条件的触发和服务器的响应。
他们可以编写容错性更强的测试脚本，允许某些辅助性操作（如环境设置或清理）失败而不影响核心测试逻辑的执行。
在允许失败的代码块中，他们可以通过检查 $mysql_errno 的值，有条件地执行后续的测试步骤或断言，从而对错误发生后的状态进行更细致的验证。
这种能力对于构建能够覆盖各种复杂场景和异常路径的健壮测试用例来说至关重要。


流程控制流程控制命令允许在 .test 文件中根据条件执行或重复执行某些命令块，从而实现更复杂的测试逻辑。

if (<表达式>)... end / }

目的与功能：if 命令用于实现条件执行。它计算括号内的 <表达式>，如果表达式的结果为真（在 mysqltest 上下文中，通常意味着非零数字或非空字符串，具体取决于表达式的类型），则执行 if 语句和对应的 end（或闭合花括号 }）之间的命令块 4。需要注意的是，mysqltest 的 if 结构比较简单，它不支持 else 或 elseif 子句 4。如果需要实现更复杂的条件分支，可能需要嵌套 if 语句或结合变量和 skip 命令来模拟。
表达式可以是一个 mysqltest 变量（如 $my_var），一个返回数字或布尔结果的函数调用，或者是一个用反引号 ` 包围的 SQL 查询（该查询应返回单行单列的标量值，此值将作为表达式的结果）。
语法格式：有两种等效的语法格式：

if (<表达式>) { <命令列表> }
if (<表达式>) <命令列表> end
其中 <命令列表> 是一条或多条 mysqltest 命令或 SQL 语句 4。


实用范例：
Code snippetlet $my_flag = 1;
if ($my_flag) {
  --echo "The flag is set (non-zero)."
}

let $user_count = `SELECT COUNT(*) FROM mysql.user WHERE User = 'test_user'`;
if ($user_count > 0) {
  --echo "User 'test_user' exists."
  DROP USER 'test_user'@'localhost';
}

if (`SELECT @@global.log_bin`) {
  --echo "Binary logging is enabled."
  --source include/check_binlog_format.inc
}
end # Corresponds to the last if

需要明确区分 mysqltest 的 if 命令与 SQL 语言自身的 IF 语句（如在存储过程中使用，见 18）或 IF() 函数。前者是测试脚本语言的控制结构，后者是数据库服务器执行的逻辑。



while (<表达式>)... end / }

目的与功能：while 命令用于实现循环执行。只要括号内的 <表达式> 计算结果为真（非零），while 和对应的 end（或 }）之间的命令块就会被重复执行 4。与 if 命令类似，表达式可以是变量、函数调用或标量 SQL 查询。必须确保循环体内部有逻辑能够最终改变表达式的值使其为假，否则会导致无限循环。
语法格式：

while (<表达式>) { <命令列表> }
while (<表达式>) <命令列表> end 4。


实用范例：
Code snippetlet $counter = 3;
--echo "Starting while loop..."
while ($counter > 0) {
  --echo "Counter is: $counter"
  # Simulate some work or check
  CREATE TABLE t_loop_$counter (a INT);
  INSERT INTO t_loop_$counter VALUES ($counter);
  dec $counter; # Decrement the counter to eventually exit the loop
}
--echo "While loop finished."
SHOW TABLES LIKE 't_loop_%';
DROP TABLE t_loop_1, t_loop_2, t_loop_3;




变量操作mysqltest 允许定义和使用变量来存储临时的值，这些值可以在测试脚本的不同部分被引用或修改，从而增强了测试的动态性和灵活性。

let $<变量名> = <值>

目的与功能：let 命令是 mysqltest 中最主要的变量赋值方式。它将 <值> 赋给以 $ 开头的 <变量名> 4。变量名不能包含空格或等号 =。<值> 可以是一个直接的字面量（如数字或字符串）、另一个 mysqltest 变量的引用，或者是一些特定内置函数（如 query_get_value）的返回值。let 命令也可以用于设置环境变量，此时变量名前不加 $ 符号（例如 let MY_CUSTOM_ENV_VAR = some_value;）4。
query_get_value(<查询语句>, <列名>, <行号>) 是一个常用的与 let 结合的函数，它执行指定的 <查询语句>，并从结果集的指定 <行号>（从1开始计数）和 <列名> 中提取单个值赋给变量。
语法格式：let $<变量名> = <值> 4。
实用范例：
Code snippetlet $test_name = "Variable Test";
let $max_connections = 151;
let $tmp_file_path = $MYSQL_TMP_DIR/my_data.txt;

--echo "Test Name: $test_name"
--echo "Max Connections: $max_connections"
--echo "Temp File Path: $tmp_file_path"

CREATE TABLE t_vars_example (id INT, name VARCHAR(50), value INT);
INSERT INTO t_vars_example VALUES (1, 'Version', `SELECT VERSION()`);
INSERT INTO t_vars_example VALUES (2, 'RowCount', 100);

let $db_version = query_get_value(SELECT name FROM t_vars_example WHERE id = 1, name, 1);
let $initial_row_count = query_get_value(SELECT value FROM t_vars_example WHERE id = 2, value, 1);

--echo "Database Version (from table): $db_version"
--echo "Initial Row Count (from table): $initial_row_count"

DROP TABLE t_vars_example;

需要注意，mysqltest 中的变量（如 $test_name）与 MySQL SQL 层面定义的变量（如用户定义变量 @sql_var 或存储过程中的局部变量，如 20 所述）在作用域、声明方式和使用语法上是不同的。mysqltest 变量主要服务于测试脚本自身的逻辑控制。



inc $<变量名> / dec $<变量名>

目的与功能：这两个命令分别用于对具有数字值的 mysqltest 变量进行自增（加1）和自减（减1）操作 4。如果变量未初始化或不包含有效的数字值，其行为可能是未定义的或导致错误。
语法格式：inc $<变量名>; 和 dec $<变量名>; 4。
实用范例：
Code snippetlet $loop_iterator = 0;
--echo "Initial iterator value: $loop_iterator"

inc $loop_iterator;
--echo "After first increment: $loop_iterator" # Output: 1

inc $loop_iterator;
inc $loop_iterator;
--echo "After two more increments: $loop_iterator" # Output: 3

dec $loop_iterator;
--echo "After one decrement: $loop_iterator" # Output: 2




执行控制这些命令用于管理测试脚本的执行流程，例如暂停、包含其他脚本文件、动态执行语句以及与外部shell环境交互。

--sleep <秒数>

目的与功能：sleep 命令使 mysqltest 客户端暂停其执行指定的 <秒数> 4。这个时间值可以是一个整数，也可以是包含小数部分的值，以实现亚秒级的暂停。此命令常用于等待某些异步操作完成，例如等待复制延迟赶上、等待后台进程启动或完成其工作，或者简单地在快速连续的操作之间引入一个短暂的延迟以观察系统行为。
语法格式：sleep <秒数>; 4。
实用范例：
Code snippet--echo "Starting a background operation (simulated)..."
--exec_in_background $MYSQL_DUMP test > $MYSQL_TMP_DIR/dump.sql
--echo "Waiting for 3.5 seconds for the dump to likely complete..."
sleep 3.5;
--echo "Checking if dump file was created..."
--file_exists $MYSQL_TMP_DIR/dump.sql
if ($return == 0) {
  --echo "Dump file found."
  --remove_file $MYSQL_TMP_DIR/dump.sql
}

需要区分 mysqltest 的 sleep 命令与 SQL 函数 SLEEP() 8。前者是 mysqltest 客户端的行为，暂停测试脚本的执行；后者是 MySQL 服务器执行的函数，会使执行该 SLEEP() 函数的服务器线程暂停。讨论了 mysqltest 客户端的一个命令行选项 --sleep=N，它可以覆盖测试文件中所有 sleep 命令的休眠时间。



--source <文件名>

目的与功能：--source (或简写为 source) 命令指示 mysqltest 从指定的 <文件名> 中读取并执行其中包含的测试命令和 SQL 语句，就如同这些命令直接写在当前文件中一样 3。这是一种实现测试脚本模块化和代码复用的关键机制。通用的设置脚本（如创建标准用户、加载公共数据集）、清理脚本或复杂的、可在多个测试用例中重复使用的测试序列，都可以被封装在单独的文件中（通常以 .inc 为扩展名，并放在 include 目录下），然后通过 --source 命令在需要时引入。mysqltest 支持嵌套的 --source 调用，但通常有一个最大嵌套层数限制（例如，4 提到的是16层）。
语法格式：--source <文件名> 或 source <文件名> 4。
实用范例：
假设有一个文件 include/common_setup.inc 内容如下：
Code snippet# include/common_setup.inc
CREATE TABLE IF NOT EXISTS common_table (id INT, data VARCHAR(100));
TRUNCATE TABLE common_table;
INSERT INTO common_table VALUES (1, 'Common Data 1'), (2, 'Common Data 2');

在主测试文件中可以这样使用：
Code snippet--echo "Sourcing the common setup script..."
--source include/common_setup.inc
--echo "Common setup complete. Verifying common_table:"
SELECT * FROM common_table;





eval <语句>

目的与功能：eval 命令提供了一种在执行 SQL 语句或 mysqltest 命令之前先进行变量替换的机制。它会获取 <语句> 字符串，将其中的所有 mysqltest 变量（如 $var_name）替换为其当前值，然后将替换后的结果字符串作为一条新的语句来执行。这对于动态构建 SQL 查询或命令非常有用，例如，当表名、列名或查询条件的一部分存储在变量中时。
语法格式：eval <包含变量的语句>; 。
实用范例：
Code snippetlet $target_database = "dynamic_db";
let $target_table = "user_stats";
let $user_id_column = "user_id";
let $metric_column = "login_count";

eval CREATE DATABASE IF NOT EXISTS $target_database;
eval USE $target_database;
eval CREATE TABLE IF NOT EXISTS $target_table ($user_id_column INT PRIMARY KEY, $metric_column INT DEFAULT 0);

let $current_user = 101;
let $new_login_count = 5;
eval INSERT INTO $target_table ($user_id_column, $metric_column) VALUES ($current_user, $new_login_count)
  ON DUPLICATE KEY UPDATE $metric_column = $metric_column + VALUES($metric_column);

eval SELECT $metric_column FROM $target_table WHERE $user_id_column = $current_user;





exec <shell命令>

目的与功能：exec (或 --exec) 命令允许 mysqltest 执行一个外部的 shell 命令，并将该命令的标准输出包含在测试结果中（除非结果日志被禁用）4。在执行 shell 命令之前，命令字符串中的 mysqltest 变量同样会进行替换。这为与操作系统交互、调用其他工具（如 mysqldump, mysqladmin）、检查文件系统状态或执行无法通过 SQL 或 mysqltest 内部命令完成的任务提供了途径。
语法格式：--exec <shell命令及其参数> 或 exec <shell命令及其参数> 4。
实用范例：
Code snippetlet $backup_file = "$MYSQL_TMP_DIR/test_backup.sql";
--echo "Attempting to dump the 'test' database to $backup_file"
--exec $MYSQL_DUMP --databases test --result-file=$backup_file --user=root
# The output of mysqldump (if any to stdout/stderr) might appear here if not redirected

--echo "Checking if the backup file was created and is not empty:"
--file_exists $backup_file
if ($return == 0) {
  --exec ls -lh $backup_file # Show file details
  --remove_file $backup_file
  --echo "Backup file removed."
} else {
  --echo "Backup file NOT found."
}




结果格式化这些命令用于控制 SQL 查询结果在 .result 文件中的显示方式，以及对结果内容进行转换，以确保测试的确定性和可读性。

--query_vertical <语句> / query_horizontal <语句>

目的与功能：这两个命令分别用于以垂直格式或水平格式执行紧随其后的单条 SQL <语句> 并显示其结果 4。水平格式是传统的表格视图，列名在上方，数据行在下方。垂直格式则将每一行数据显示为一组“列名: 值”的对，每条记录之间通常有分隔符，类似于 mysql 命令行客户端中使用 \G 终止符的效果 9。这种格式在处理包含很多列或列内容很宽的表时特别有用，可以避免水平显示时的换行和混乱。这些命令的效果仅限于其后的那一条语句，不会改变后续语句的默认显示格式。
语法格式：query_vertical <SQL语句>; 和 query_horizontal <SQL语句>; 4。
实用范例：
Code snippetCREATE TABLE t_display_format (
  id INT,
  name VARCHAR(50),
  description TEXT,
  created_ts TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
INSERT INTO t_display_format (id, name, description) VALUES
  (1, 'First Item', 'This is a detailed description for the first item.'),
  (2, 'Second Item', 'Another description, possibly longer, for the second item.');

--echo "Default (horizontal) output for a regular SELECT:"
SELECT * FROM t_display_format WHERE id = 1;

--echo "Vertical output for the same query using query_vertical:"
query_vertical SELECT * FROM t_display_format WHERE id = 1;

--echo "Horizontal output for another query using query_horizontal:"
query_horizontal SELECT id, name FROM t_display_format WHERE id = 2;

DROP TABLE t_display_format;





vertical_results / horizontal_results

目的与功能：与 query_vertical 和 query_horizontal 不同，vertical_results 和 horizontal_results 命令用于设置后续所有 SQL 查询结果的 默认 显示格式 4。mysqltest 的初始默认显示格式是水平的 (horizontal_results 的状态)。一旦执行了 vertical_results;，之后所有未被 query_horizontal 特别指定的查询都将以垂直格式显示其结果，直到遇到 horizontal_results; 命令或测试结束。
语法格式：vertical_results; 和 horizontal_results; 4。
实用范例：
Code snippetCREATE TABLE t_default_display (pk INT PRIMARY KEY, val1 VARCHAR(20), val2 INT);
INSERT INTO t_default_display VALUES (10, 'Alpha', 100), (20, 'Beta', 200);

--echo "Switching to vertical results mode by default."
vertical_results;

--echo "This SHOW CREATE TABLE will be vertical:"
SHOW CREATE TABLE t_default_display;
--echo "This SELECT will also be vertical:"
SELECT * FROM t_default_display WHERE pk = 10;

--echo "Switching back to horizontal results mode by default."
horizontal_results;
--echo "This SELECT will now be horizontal again:"
SELECT * FROM t_default_display WHERE pk = 20;

DROP TABLE t_default_display;





replace_regex /模式/替换/[i]

目的与功能：replace_regex 命令用于对紧随其后的下一条语句所产生的输出结果（在写入 .result 文件之前）进行基于正则表达式的查找和替换 4。这对于处理那些在不同测试运行中可能会动态变化的内容（例如时间戳、自动生成的ID、文件路径、内存地址、浮点数精度差异等）至关重要，目的是确保 .result 文件的稳定性和可比较性。通过将这些易变部分替换为固定的占位符（如 <TIMESTAMP>、<UUID>、<PATH>)，可以使得测试结果在不同环境或不同时间运行时保持一致，从而避免因这些非功能性差异导致的测试失败。可选的 i标志表示正则表达式匹配时不区分大小写。可以指定多个正则表达式进行替换。
语法格式：replace_regex /<正则表达式模式>/<替换字符串>/[i] 4。
实用范例：
Code snippetCREATE TABLE t_dynamic_data (
  id INT AUTO_INCREMENT PRIMARY KEY,
  event_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  temp_file_path VARCHAR(255)
);
INSERT INTO t_dynamic_data (temp_file_path) VALUES ('/tmp/mysqltest_file_12345.tmp');
INSERT INTO t_dynamic_data (temp_file_path) VALUES ('/var/tmp/mysql_test_another.dat');

--echo "Original data with dynamic parts:"
--disable_result_log # Hide original output to avoid clutter in actual result
SELECT * FROM t_dynamic_data;
--enable_result_log

--echo "Data after regex replacements for consistency:"
--replace_regex /[0-9]{4}-[0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2}:[0-9]{2}/<DATETIME>/
--replace_regex /AUTO_INCREMENT=[0-9]+/AUTO_INCREMENT=<AI_VALUE>/
--replace_regex /[^ ]+\/(mysqltest_file_|mysql_test_another)[^ ]+/--normalized--path--/
SELECT * FROM t_dynamic_data;

--replace_regex /AUTO_INCREMENT=[0-9]+/AUTO_INCREMENT=<AI_VALUE>/
SHOW CREATE TABLE t_dynamic_data;

DROP TABLE t_dynamic_data;

6 中也强调了使用此类替换机制来处理环境特定内容，以保证测试的确定性。



sorted_result

目的与功能：对于那些不使用 ORDER BY 子句的 SELECT 查询，数据库系统通常不保证返回结果行的顺序。这种不确定性会导致测试在不同次运行时产生表面上不同的 .result 文件，即使实际数据内容是正确的，从而导致测试误判失败。sorted_result 命令解决了这个问题，它指示 mysqltest 在捕获到下一条 SQL 语句（通常是 SELECT）的结果集后，先对整个结果集进行排序，然后再将排序后的结果与期望的 .result 文件内容进行比较 4。这确保了只要数据内容一致，无论原始顺序如何，测试都能通过。
语法格式：sorted_result; 4。
实用范例：
Code snippetCREATE TABLE t_unsorted (id INT, category CHAR(1), value INT);
INSERT INTO t_unsorted VALUES (3, 'A', 100), (1, 'B', 200), (2, 'A', 300);

--echo "Query without ORDER BY, result order might vary:"
--disable_result_log
SELECT * FROM t_unsorted;
--enable_result_log

--echo "Same query, but with sorted_result to ensure deterministic output for comparison:"
--sorted_result
SELECT * FROM t_unsorted;
# Expected output in.result file (after sorting by mysqltest, typically lexicographical on all columns):
# id    category        value
# 1     B       200
# 2     A       300
# 3     A       100

DROP TABLE t_unsorted;



结果格式化和替换命令，如 replace_regex、sorted_result 以及其他类似的命令（例如 replace_column，用于替换特定列中的值；replace_result，用于替换整个结果中的精确字符串匹配），是确保测试确定性 (determinism) 和可移植性 (portability) 的核心工具。数据库查询的输出天然包含一些不确定或动态变化的元素：没有 ORDER BY 的查询，其行的返回顺序可能因执行计划、内部数据结构或并发活动而异；时间戳、临时文件名、服务器生成的唯一ID（如UUID或自增ID的当前值）、某些错误消息中的特定参数等，都会随执行环境（操作系统、文件系统布局、时区）和执行时间的不同而变化。
如果不对这些动态或不确定的部分进行规范化处理，那么为一次成功运行生成的 .result 文件，在下一次（可能在不同机器、不同时间或稍有不同的配置下）运行时，即使被测功能本身完全正确，也极有可能因为这些非本质的差异而导致测试失败。这将产生大量的“误报 (false positives/negatives)”，极大地浪费开发人员和测试人员的调试时间，并降低对测试套件可靠性的信任。
因此，mysqltest 提供这些结果转换命令的目的，就是为了消除测试结果中的这种“噪音”，允许测试编写者定义一种规范化的、与环境和时间无关的预期输出形式。通过将易变部分替换为固定的占位符，或者通过对结果集进行排序，可以确保只要被测功能的核心逻辑正确，测试就能在各种条件下稳定地通过。这使得 .result 文件更加健壮和易于维护，测试用例也更能专注于验证其设计的核心功能点，而不是受困于外部环境的细微波动。

连接管理尽管用户查询未将连接管理作为主要关注点，但这些命令是 .test 文件不可或缺的一部分，因为它们控制着与一个或多个 MySQL 服务器实例的交互。
connect (<名称>, <主机>, <用户>, <密码>, <数据库> [, <端口> [, <套接字> [, <选项>]]]): 此命令用于建立一个新的到 MySQL 服务器的连接，或重新连接一个已存在的同名连接。参数包括连接的逻辑名称（用于后续引用）、主机名、用户名、密码、默认数据库，以及可选的端口号、套接字文件路径和连接选项（如 SSL, COMPRESS）4。
disconnect <连接名>: 关闭由 <连接名> 标识的现有连接 4。
connection <连接名>: 将当前的活动连接切换到由 <连接名> 标识的已打开连接。default 是 mysqltest 启动时建立的初始连接的名称 4。
这些命令对于测试需要多个并发连接、涉及不同用户权限、或模拟复制拓扑（主从连接）的场景至关重要。文件操作mysqltest 还提供了一组用于在测试执行期间与文件系统交互的命令。
write_file <文件名> [终止符]: 将此命令之后直到指定终止符（默认为 EOF）之间的所有行内容写入到指定的文件中。如果文件已存在，通常会报错 4。
append_file <文件名> [终止符]: 与 write_file 类似，但如果文件已存在，则将内容追加到文件末尾；如果文件不存在，则创建文件 4。
cat_file <文件名>: 读取指定文件的内容，并将其输出到测试结果中 4。
remove_file <文件名> [retry]: 删除指定的文件。可选的 retry 参数允许在某些瞬时失败（如文件被占用）时进行重试 4。
这些文件操作命令常用于准备测试数据（例如，通过 write_file 创建包含 LOAD DATA INFILE 所需数据的文本文件）、从外部文件加载 SQL 脚本（尽管 --source 更常用于此目的）、检查服务器生成的日志文件内容，或在测试结束时清理产生的临时文件。四、 编写高效 .test 文件的实践建议编写高质量的 .test 文件对于维护一个健壮、可靠且易于管理的 MySQL 测试套件至关重要。以下是一些基于所研究材料和通用测试原则的实践建议。创建确定性、可维护测试用例的技巧
确保输出顺序的确定性：对于返回多行结果的查询，除非测试的目的就是验证不确定顺序下的行为，否则应始终在 SELECT 语句中使用 ORDER BY 子句来明确指定结果的排序。如果由于某种原因无法在 SQL层面保证顺序（例如，测试一个不接受 ORDER BY 的 SHOW 命令），则应在 mysqltest 层面使用 sorted_result 命令，指示 mysqltest 在比较结果前先对实际输出进行排序 4。
处理动态和环境特定的数据：测试结果中经常会包含一些随时间或环境变化的内容，如时间戳、自动生成的ID、服务器路径、错误消息中的特定参数等。为了避免这些非功能性差异导致测试失败，应积极使用 replace_regex、replace_column 或 replace_result 等命令，将这些动态部分替换为固定的占位符或通用模式 4。例如，6 明确建议使用 --replace_regex 来处理环境特定的内容。
清晰一致的命名：为测试中创建的数据库对象（如表、视图、存储过程）以及 mysqltest 变量使用清晰、描述性且一致的命名约定 3。这有助于提高测试脚本的可读性和可维护性。
保证测试的可重复性：每个测试用例应设计为可以独立且重复执行，而不受先前测试运行状态的影响。这通常意味着在测试开始时，需要清理可能存在的同名对象（例如，使用 DROP TABLE IF EXISTS...;），并在测试结束时，清理该测试所创建的所有数据库对象和临时文件 3。
充分的注释：使用 --echo 命令在 .test 文件和 .result 文件中添加足够的注释，解释测试的目的、关键步骤的逻辑、以及为什么某些特定的检查是必要的 6。良好的注释对于他人（以及未来的自己）理解和维护测试用例至关重要。
利用 --source 实现测试脚本的模块化
封装通用逻辑：将通用的设置序列（例如，创建一组标准用户、加载一套公共的测试数据、配置特定的服务器变量）、通用的清理逻辑，或者一些可以在多个测试用例中重复使用的复杂测试步骤序列，封装到单独的 .inc 文件中。然后，在主测试文件中通过 --source <文件名> 命令来引入和执行这些模块化的脚本 3。
提高可读性和可维护性：模块化可以显著提高大型测试套件的可读性和可维护性。它减少了代码冗余，使得对通用部分的修改只需要在一个地方进行，并使得主测试文件更专注于其特定的测试目标。
结果验证和错误处理的最佳实践
明确断言预期结果：仅仅检查一个操作是否成功执行（即没有抛出错误）通常是不够的。对于关键操作，应该明确验证其产生的结果是否符合预期，例如，查询返回的行数、特定列的值、或操作影响的记录数。
优先使用符号错误名：在使用 --error 命令声明预期错误时，应优先使用符号错误名称（如 ER_NO_SUCH_TABLE），而不是数字错误代码或 SQLSTATE 值 6。符号错误名更具可读性，且不易因 MySQL 版本更迭导致错误代码数值变化而失效。
审慎使用 disable_abort_on_error：虽然 disable_abort_on_error 在某些情况下（如执行可选的清理操作或测试特定的错误恢复流程）很有用，但不应滥用。过度使用可能掩盖测试脚本或被测功能中潜在的问题。应仅在确实需要忽略某些非关键错误，或者需要捕获错误码并基于此进行后续逻辑判断时才使用它 7。
全面的错误和边界条件测试：在为新功能编写测试用例时，不仅要覆盖其正常的、预期的使用路径（“happy path”），还必须设计测试用例来验证其在各种错误输入、非法操作和边界条件下的行为是否正确和健壮 6。这包括测试服务器是否能返回正确的错误代码、是否能优雅地处理无效数据，以及在资源限制等极端情况下的表现。
遵循这些良好的 .test 文件编写实践，对于构建和维护一个高质量、高效率的 MySQL 测试套件具有直接且深远的影响。正如 6 中专门有一节讨论“良好测试用例的实践”所强调的那样，不确定性的测试（例如，依赖于未排序的查询输出或包含易变数据的结果）会导致测试结果的波动，产生误报或漏报，从而浪费大量的调试时间并降低对测试套件的信任度。同样，那些缺乏注释、结构混乱、难以理解和维护的测试用例，会显著增加新功能测试的开发成本，以及在修复缺陷时进行回归验证的难度。因此，通过确保测试的确定性（使用排序和替换机制）、实现模块化（利用 --source）、采用清晰的错误处理策略（恰当使用 --error 和 disable_abort_on_error），并编写易于理解的代码（通过命名和注释），可以构建一个可靠、高效且易于扩展的测试基础。这不仅能更有效地保障 MySQL 服务器的质量，也能提升整个开发和测试团队的工作效率。五、 结论与展望总结 mysql-test-runner.pl 和 .test 文件标记的核心功能mysql-test-runner.pl (MTR) 作为 MySQL 测试套件的核心驱动程序，通过调用 mysqltest 工具来执行 .test 文件中定义的测试用例，扮演着自动化测试流程、管理测试环境和验证测试结果的关键角色。.test 文件本身则通过一套专门的命令标记（mysqltest 语言）提供了强大的测试能力。这些标记使得测试工程师能够精确控制 SQL 语句的执行、管理数据库连接、实现复杂的测试流程逻辑（通过条件和循环）、灵活处理和验证预期及意外的错误、格式化和规范化测试输出以确保确定性，并与外部环境进行交互。诸如 --echo, --error, --let, if, while, --sleep, --source, --query_vertical, --query_horizontal, disable_abort_on_error, replace_regex 和 sorted_result 等命令，共同构成了一个功能完备的领域特定语言，专用于高效、准确地测试 MySQL 服务器的各项功能。提供进一步学习和探索的资源方向对于希望进一步深入理解 MySQL 测试框架和 mysqltest 命令的开发人员和测试工程师，以下资源将提供宝贵的信息：
MySQL 官方文档：应始终查阅最新版本的 MySQL 官方文档中关于 mysql-test-run.pl 和 mysqltest 语言参考的部分。这些文档通常包含了最权威和最全面的命令说明、选项解释和使用示例 11。
MySQL Server 源代码库：直接研究 MySQL Server 源代码发行版中的 mysql-test/ 目录，特别是其下的 t/（测试用例文件）和 include/（可包含的脚本文件）子目录，是学习实际测试用例编写技巧和高级用法的最佳途径 3。通过分析现有的大量测试用例，可以了解各种命令是如何组合使用以测试特定功能、处理复杂场景以及遵循最佳实践的。
持续学习和实践这些测试技术，将有助于构建更强大、更可靠的 MySQL 应用程序和扩展，并为 MySQL 社区的整体质量做出贡献。