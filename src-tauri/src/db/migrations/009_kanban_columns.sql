CREATE TABLE kanban_columns (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  color TEXT NOT NULL DEFAULT '#1677ff',
  subtitle TEXT,
  sort_order INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL
);

ALTER TABLE todos ADD COLUMN kanban_column_id INTEGER REFERENCES kanban_columns(id) ON DELETE SET NULL;

INSERT INTO kanban_columns (name, color, subtitle, sort_order, created_at) VALUES
  ('重要不紧急', '#1677ff', '计划安排', 0, datetime('now')),
  ('重要且紧急', '#cf1322', '立即处理', 1, datetime('now')),
  ('不重要不紧急', '#8c8c8c', '可暂缓', 2, datetime('now')),
  ('不重要但紧急', '#d48806', '快速处理', 3, datetime('now'));

UPDATE todos SET kanban_column_id = CASE quadrant
  WHEN 1 THEN 2
  WHEN 2 THEN 1
  WHEN 3 THEN 4
  WHEN 4 THEN 3
  ELSE NULL
END WHERE quadrant IS NOT NULL;

ALTER TABLE todos DROP COLUMN quadrant;
