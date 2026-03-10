import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import type { TActivity } from "./types";
import { handleFetchResponse } from "./util";

export function useActivity(sourceId?: number) {
	return useQuery<TActivity[]>({
		queryKey: ["activity", sourceId],
		queryFn: () =>
			fetch("/api/activity" + (sourceId ? `/${sourceId}` : "")).then(
				handleFetchResponse("Error fetching activities"),
			),
	});
}

export function useRecheckRSS() {
	const queryClient = useQueryClient();

	return useMutation({
		mutationFn: () =>
			fetch("/api/recheck", {
				method: "POST",
			}).then(handleFetchResponse("Error rechecking rss")),
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ["activity"] });
		},
	});
}

export function useClearActivities() {
	const queryClient = useQueryClient();

	return useMutation({
		mutationFn: (num: number) =>
			fetch(`/api/activity${num < 1 ? "" : `/${num}`}`, {
				method: "DELETE",
			}).then(handleFetchResponse("Error clearing activities")),
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ["activity"] });
		},
	});
}
