ALTER TABLE todos ADD COLUMN tags_text TEXT DEFAULT '';

CREATE VIRTUAL TABLE IF NOT EXISTS todos_fts USING fts5(
  title,
  content_text,
  tags_text,
  content='todos',
  content_rowid='id'
);

CREATE TRIGGER IF NOT EXISTS todos_ai AFTER INSERT ON todos BEGIN
  INSERT INTO todos_fts(rowid, title, content_text, tags_text)
  VALUES (new.id, new.title, new.content_text, new.tags_text);
END;

CREATE TRIGGER IF NOT EXISTS todos_ad AFTER DELETE ON todos BEGIN
  INSERT INTO todos_fts(todos_fts, rowid, title, content_text, tags_text)
  VALUES ('delete', old.id, old.title, old.content_text, old.tags_text);
END;

CREATE TRIGGER IF NOT EXISTS todos_au AFTER UPDATE ON todos BEGIN
  INSERT INTO todos_fts(todos_fts, rowid, title, content_text, tags_text)
  VALUES ('delete', old.id, old.title, old.content_text, old.tags_text);
  INSERT INTO todos_fts(rowid, title, content_text, tags_text)
  VALUES (new.id, new.title, new.content_text, new.tags_text);
END;

INSERT INTO todos_fts(todos_fts) VALUES('rebuild');
