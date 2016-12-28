CREATE TABLE IF NOT EXISTS `stories` (
  `id` INTEGER PRIMARY KEY,
  `title` VARCHAR,
  `guid` VARCHAR,
  `pub_date` VARCHAR,
  `description` VARCHAR,
  `feed_url` VARCHAR NOT NULL
);

CREATE INDEX IF NOT EXISTS `stories_guid_feed_url_index` ON `stories` (`guid`, `feed_url`);
