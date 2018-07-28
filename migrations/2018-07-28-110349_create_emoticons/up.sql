CREATE TABLE emoticons (
  id bigint PRIMARY KEY,
  code character varying NOT NULL
);

CREATE INDEX tgrm_index_emoticons_on_code ON emoticons USING GIN (code gin_trgm_ops);

