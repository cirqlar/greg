import dayjs from "dayjs";

export function formatDate(date: string) {
	return dayjs(new Date(date)).format("h:mm a DD/MM/YYYY");
}