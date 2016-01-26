CREATE TABLE `results` (
  `id` binary(16) NOT NULL,
  `command` varchar(255) COLLATE utf8_unicode_ci DEFAULT NULL,
  `cwd` varchar(255) COLLATE utf8_unicode_ci DEFAULT NULL,
  `stdout` text COLLATE utf8_unicode_ci DEFAULT NULL,
  `stderr` text COLLATE utf8_unicode_ci DEFAULT NULL,
  `started_at` datetime COLLATE utf8_unicode_ci DEFAULT NULL,
  `finished_at` datetime COLLATE utf8_unicode_ci DEFAULT NULL,
  `status` smallint(4) DEFAULT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8 COLLATE=utf8_unicode_ci;
