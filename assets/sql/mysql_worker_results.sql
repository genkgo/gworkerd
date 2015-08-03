CREATE TABLE `results` (
  `id` binary(16) NOT NULL,
  `command` varchar(255) COLLATE utf8_unicode_ci DEFAULT NULL,
  `cwd` varchar(255) COLLATE utf8_unicode_ci DEFAULT NULL,
  `stdout` varchar(45) COLLATE utf8_unicode_ci DEFAULT NULL,
  `stderr` varchar(45) COLLATE utf8_unicode_ci DEFAULT NULL,
  `status` smallint(4) DEFAULT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8 COLLATE=utf8_unicode_ci;