import dayjs from "dayjs";

export function formatDate(date: string) {
	return dayjs(new Date(date)).format("HH:mm a DD/MM/YYYY");
}