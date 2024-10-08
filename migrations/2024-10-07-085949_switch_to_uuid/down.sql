-- This file should undo anything in `up.sql`
-- Your SQL goes here
PRAGMA foreign_keys = OFF;
BEGIN;
CREATE TABLE `categories_new`(
	`id` INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    `old_id` BINARY NOT NULL,
	`label` TEXT NOT NULL,
	`from_age` INTEGER NOT NULL,
	`to_age` INTEGER NOT NULL,
	`male` BOOL NOT NULL,
	`start_id` INTEGER NOT NULL,
	FOREIGN KEY (`start_id`) REFERENCES `starts`(`id`) ON DELETE CASCADE
);

CREATE TABLE `races_new`(
	`id` INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    `old_id` BINARY NOT NULL,
	`name` TEXT NOT NULL,
	`competition_id` INTEGER NOT NULL,
	FOREIGN KEY (`competition_id`) REFERENCES `competitions`(`id`) ON DELETE CASCADE
);

CREATE TABLE `participants_new`(
	`id` INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    `old_id` BINARY NOT NULL,
	`last_name` TEXT NOT NULL,
	`first_name` TEXT NOT NULL,
	`club` TEXT,
	`category_id` INTEGER NOT NULL,
	`consent_agb` BOOL NOT NULL,
	`birth_year` INTEGER NOT NULL,
	FOREIGN KEY (`category_id`) REFERENCES `categories`(`id`) ON DELETE CASCADE
);

CREATE TABLE `competitions_new`(
	`id` INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    `old_id` BINARY NOT NULL,
	`name` TEXT NOT NULL,
	`description` TEXT NOT NULL,
	`date` DATE NOT NULL,
	`location` TEXT NOT NULL,
	`announcement` TEXT NOT NULL
);

CREATE TABLE `starts_new`(
	`id` INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    `old_id` BINARY NOT NULL,
	`name` TEXT NOT NULL,
	`time` TIMESTAMP NOT NULL,
	`race_id` INTEGER NOT NULL,
	FOREIGN KEY (`race_id`) REFERENCES `races`(`id`) ON DELETE CASCADE
);

CREATE TABLE `special_categories_new`(
	`id` INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    `old_id` BINARY NOT NULL,
	`short_name` TEXT NOT NULL,
	`name` TEXT NOT NULL,
	`race_id` INTEGER NOT NULL,
	FOREIGN KEY (`race_id`) REFERENCES `races`(`id`) ON DELETE CASCADE
);

CREATE TABLE `participants_in_special_category_new`(
	`participant_id` BINARY NOT NULL REFERENCES participants(id) ON DELETE CASCADE,
	`special_category_id` BINARY NOT NULL REFERENCES special_categories(id) ON DELETE CASCADE,
	PRIMARY KEY(`participant_id`, `special_category_id`)
);

CREATE TABLE `users_new`(
	`id` INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
	`name` TEXT UNIQUE NOT NULL,
	`password` TEXT NOT NULL
);

INSERT INTO `competitions_new` (`old_id`, `name`, `description`, `date`, `location`, `announcement`)
SELECT `id`, `name`, `description`, `date`, `location`, `announcement` FROM `competitions`;

INSERT INTO `races_new` (`old_id`, `name`, `competition_id`)
SELECT `r`.`id`, `r`.`name`, `c`.`id` FROM `races` AS `r`
INNER JOIN `competitions_new` AS `c` ON `r`.`competition_id` = `c`.`old_id`;

INSERT INTO `starts_new` (`old_id`, `name`, `time`, `race_id`)
SELECT `s`.`id`, `s`.`name`, `s`.`time`, `r`.`id` FROM `starts` AS `s`
INNER JOIN `races_new` AS `r` ON `s`.`race_id` = `r`.`old_id`;

INSERT INTO `categories_new` (`old_id`, `label`, `from_age`, `to_age`, `male`, `start_id`)
SELECT `c`.`id`, `c`.`label`, `c`.`from_age`, `c`.`to_age`, `c`.`male`, `s`.`id` FROM `categories` AS `c`
INNER JOIN `starts_new` AS `s` ON `c`.`start_id` = `s`.`old_id`;

INSERT INTO `participants_new` (`old_id`, `first_name`, `last_name`, `club`, `category_id`, `consent_agb`, `birth_year`)
SELECT `p`.`id`, `p`.`first_name`, `p`.`last_name`, `p`.`club`, `c`.`id`, `p`.`consent_agb`, `p`.`birth_year` FROM `participants` AS `p`
INNER JOIN `categories_new` AS `c` ON `p`.`category_id` = `c`.`old_id`;

INSERT INTO `special_categories_new` (`old_id`, `short_name`, `name`, `race_id`)
SELECT `s`.`id`, `s`.`short_name`, `s`.`name`, `r`.`id` FROM `special_categories` AS `s`
INNER JOIN `races_new` AS `r` ON `s`.`race_id` = `r`.`old_id`;

INSERT INTO `participants_in_special_category_new` (`participant_id`, `special_category_id`)
SELECT `p`.`id`, `s`.`id` FROM `participants_in_special_category_new` AS `ps`
INNER JOIN `special_categories_new` AS `s` ON `ps`.`special_category_id` = `s`.`old_id`
INNER JOIN `participants_new` AS `p` ON `ps`.`participant_id` = `p`.`old_id`;

INSERT INTO `users_new` (`name`, `password`)
SELECT `name`, `password` FROM `users`;

ALTER TABLE `competitions_new` DROP COLUMN `old_id`;
ALTER TABLE `races_new` DROP COLUMN `old_id`;
ALTER TABLE `starts_new` DROP COLUMN `old_id`;
ALTER TABLE `categories_new` DROP COLUMN `old_id`;
ALTER TABLE `participants_new` DROP COLUMN `old_id`;
ALTER TABLE `special_categories_new` DROP COLUMN `old_id`;

DROP TABLE `users`;
DROP TABLE `participants_in_special_category`;
DROP TABLE `special_categories`;
DROP TABLE `participants`;
DROP TABLE `categories`;
DROP TABLE `starts`;
DROP TABLE `races`;
DROP TABLE `competitions`;


ALTER TABLE `competitions_new` RENAME TO `competitions`;
ALTER TABLE `races_new` RENAME TO `races`;
ALTER TABLE `starts_new` RENAME TO `starts`;
ALTER TABLE `categories_new` RENAME TO `categories`;
ALTER TABLE `participants_new` RENAME TO `participants`;
ALTER TABLE `special_categories_new` RENAME TO `special_categories`;
ALTER TABLE `participants_in_special_category_new` RENAME TO `participants_in_special_category`;
ALTER TABLE `users_new` RENAME TO `users`;

END;
PRAGMA foreign_keys = ON;
