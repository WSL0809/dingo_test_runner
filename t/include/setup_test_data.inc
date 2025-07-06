# Include file for setting up test data
--echo Setting up test data from include file

CREATE TABLE include_test_data (
    id INT PRIMARY KEY,
    category VARCHAR(50),
    description TEXT
);

INSERT INTO include_test_data VALUES 
    (1, 'Category A', 'Description for item 1'),
    (2, 'Category B', 'Description for item 2'),
    (3, 'Category A', 'Description for item 3');

--echo Test data setup completed from include file