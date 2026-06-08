ALTER TABLE tags ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0;

UPDATE tags
SET sort_order = (
  SELECT COUNT(*) - 1
  FROM tags AS t2
  WHERE t2.created_at < tags.created_at
     OR (t2.created_at = tags.created_at AND t2.id <= tags.id)
);
