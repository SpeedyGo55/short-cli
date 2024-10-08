-- Add migration script here
DROP TABLE urls;
CREATE TABLE urls (
                      id VARCHAR(10) PRIMARY KEY,
                      url VARCHAR NOT NULL
);
