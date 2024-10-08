-- Your SQL goes here
CREATE TABLE `categories`(
	`id` INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
	`label` TEXT NOT NULL,
	`from_age` INTEGER NOT NULL,
	`to_age` INTEGER NOT NULL,
	`male` BOOL NOT NULL,
	`start_id` BINARY NOT NULL,
	FOREIGN KEY (`start_id`) REFERENCES `starts`(`id`) ON DELETE CASCADE
);

CREATE TABLE `races`(
	`id` INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
	`name` TEXT NOT NULL,
	`competition_id` BINARY NOT NULL,
	FOREIGN KEY (`competition_id`) REFERENCES `competitions`(`id`) ON DELETE CASCADE
);

CREATE TABLE `participants`(
	`id` INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
	`last_name` TEXT NOT NULL,
	`first_name` TEXT NOT NULL,
	`club` TEXT,
	`category_id` INTEGER NOT NULL,
	`consent_agb` BOOL NOT NULL,
	`birth_year` INTEGER NOT NULL,
	FOREIGN KEY (`category_id`) REFERENCES `categories`(`id`) ON DELETE CASCADE
);

CREATE TABLE `competitions`(
	`id` INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
	`name` TEXT NOT NULL,
	`description` TEXT NOT NULL,
	`date` DATE NOT NULL,
	`location` TEXT NOT NULL,
	`announcement` TEXT NOT NULL
);

CREATE TABLE `starts`(
	`id` INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
	`name` TEXT NOT NULL,
	`time` TIMESTAMP NOT NULL,
	`race_id` INTEGER NOT NULL,
	FOREIGN KEY (`race_id`) REFERENCES `races`(`id`) ON DELETE CASCADE
);

CREATE TABLE `special_categories`(
	`id` INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
	`short_name` TEXT NOT NULL,
	`name` TEXT NOT NULL,
	`race_id` INTEGER NOT NULL,
	FOREIGN KEY (`race_id`) REFERENCES `races`(`id`) ON DELETE CASCADE
);

CREATE TABLE `participants_in_special_category`(
	`participant_id` INTEGER NOT NULL REFERENCES participants(id) ON DELETE CASCADE,
	`special_category_id` INTEGER NOT NULL REFERENCES special_categories(id) ON DELETE CASCADE,
	PRIMARY KEY(`participant_id`, `special_category_id`)
);

CREATE TABLE `users`(
	`id` INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
	`name` TEXT UNIQUE NOT NULL,
	`password` TEXT NOT NULL
);

CREATE TABLE `session_records`(
	`id` BINARY NOT NULL PRIMARY KEY,
	`data` TEXT NOT NULL,
	`expiry_date` TEXT NOT NULL
);
