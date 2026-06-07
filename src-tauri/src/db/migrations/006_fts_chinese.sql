ALTER TABLE todos ADD COLUMN title_text TEXT DEFAULT '';
ALTER TABLE todos ADD COLUMN assignee_text TEXT DEFAULT '';

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
  tokenize='unicode61'
);

CREATE TRIGGER todos_ai AFTER INSERT ON todos BEGIN
  INSERT INTO todos_fts(rowid, title_text, content_text, tags_text, assignee_text)
  VALUES (new.id, new.title_text, new.content_text, new.tags_text, new.assignee_text);
END;

CREATE TRIGGER todos_ad AFTER DELETE ON todos BEGIN
  INSERT INTO todos_fts(todos_fts, rowid, title_text, content_text, tags_text, assignee_text)
  VALUES ('delete', old.id, old.title_text, old.content_text, old.tags_text, old.assignee_text);
END;

CREATE TRIGGER todos_au AFTER UPDATE ON todos BEGIN
  INSERT INTO todos_fts(todos_fts, rowid, title_text, content_text, tags_text, assignee_text)
  VALUES ('delete', old.id, old.title_text, old.content_text, old.tags_text, old.assignee_text);
  INSERT INTO todos_fts(rowid, title_text, content_text, tags_text, assignee_text)
  VALUES (new.id, new.title_text, new.content_text, new.tags_text, new.assignee_text);
END;
