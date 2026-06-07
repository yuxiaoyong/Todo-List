DROP TRIGGER IF EXISTS todos_ai;
DROP TRIGGER IF EXISTS todos_ad;
DROP TRIGGER IF EXISTS todos_au;

DROP TABLE IF EXISTS todos_fts;

CREATE VIRTUAL TABLE todos_fts USING fts5(
  title_text,
  content_text,
  tags_text,
  assignee_text,
  content='todos',
  content_rowid='id',
  tokenize='porter unicode61'
);

