import fs from "node:fs";
import path from "node:path";
import {
	FileMigrationProvider,
	type Kysely,
	type MigrationResultSet,
	Migrator,
} from "kysely";
import { logger } from "./log.js";

async function main(
	db: Kysely<any>,
	migrationsPath: string,
	downgrade = false,
) {
	logger.info("Migrator: checking for migrations");

	const provider = new FileMigrationProvider({
		fs: fs.promises,
		migrationFolder: migrationsPath,
		path,
	});
	const migrator = new Migrator({
		db,
		provider,
	});

	let res: MigrationResultSet;
	if (downgrade) {
		res = await migrator.migrateDown();
	} else {
		res = await migrator.migrateToLatest();
	}

	const { error, results } = res;
	const action = downgrade ? "reverted" : "applied";
	results?.forEach((it) => {
		if (it.status === "Success") {
			logger.info(`Migration "${it.migrationName}" was ${action}`);
		} else if (it.status === "Error") {
			logger.error(`Failed to execute migration "${it.migrationName}"`);
		}
	});

	if (error) {
		logger.error(error, "Failed to migrate");
		process.exit(1);
	}
}

export async function migrateDB(
	db: Kysely<any>,
	migrationsPath: string,
	downgrade = false,
) {
	try {
		await main(db, migrationsPath, downgrade);
		logger.info("Migrator finished");
	} catch (err) {
		logger.error(err, "Migration failed:");
		process.exit(1);
	}
}
