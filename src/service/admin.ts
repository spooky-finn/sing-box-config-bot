import { UserStatus } from "db/enums";
import type TelegramBot from "node-telegram-bot-api";
import { logger } from "#root/ioc.js";
import type { IUserRepo } from "#root/ports/user.js";

const InvationCmdOpCode = "invate_confirm";

export class InvationCmd {
	constructor(
		readonly userId: number,
		readonly status: UserStatus,
	) {}

	static parse(text: string) {
		const [opcode, userId, status] = text.split("_");
		if (opcode !== InvationCmdOpCode) {
			throw Error("Wrong operation code");
		}
		if (!Object.values(UserStatus).includes(status)) {
			throw Error("Invalid user status");
		}
		const uid = Number(userId);
		if (Number.isNaN(uid)) {
			throw Error("User id is NaN");
		}
		return new InvationCmd(uid, status as any);
	}

	toString() {
		return `${InvationCmdOpCode}_${this.userId}_${this.status}`;
	}
}

export class AdminService {
	constructor(
		private readonly userRepo: IUserRepo,
		readonly adminId: string,
	) {}

	isAdminCallback(msg: TelegramBot.Message): InvationCmd | false {
		if (msg.from?.id?.toString() !== this.adminId || !msg.text) {
			return false;
		}

		try {
			return InvationCmd.parse(msg.text);
		} catch (error) {
			return false;
		}
	}

	async handleAdminCallback(cmd: InvationCmd) {
		await this.userRepo.updateStatus(cmd.userId, cmd.status);
		logger.info(cmd, "adming callback handled");
	}
}
