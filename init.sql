CREATE TABLE IF NOT EXISTS `stories` (
  `id` INTEGER PRIMARY KEY,
  `title` VARCHAR NOT NULL,
  `guid` VARCHAR NOT NULL,
  `pub_date` VARCHAR,
  `content` TEXT,
  `description` VARCHAR NOT NULL,
  `feed_url` VARCHAR NOT NULL
);

CREATE INDEX IF NOT EXISTS `stories_guid_feed_url_index` ON `stories` (`guid`, `feed_url`);
