import {
	useInfiniteQuery,
	useMutation,
	useQuery,
	useQueryClient,
} from "@tanstack/react-query";

import type { TActivity } from "./types";
import { handleFetchResponse } from "./util";

export function useActivity(sourceId?: number, demo?: boolean) {
	return useQuery({
		queryKey: ["activity", sourceId, demo ?? false],
		queryFn: (): Promise<TActivity[]> =>
			fetch(
				"/api/activity" +
					(sourceId ? `/${sourceId}` : "") +
					(demo ? "?demo=true" : ""),
			).then(handleFetchResponse("Error fetching activities")),
	});
}

export function useInfiniteActivity(
	sourceId?: number,
	demo?: boolean,
	count: number = 35,
) {
	return useInfiniteQuery({
		queryKey: ["activity", sourceId, count, demo ?? false],
		queryFn: ({ pageParam }): Promise<TActivity[]> => {
			const searchParams = new URLSearchParams();
			searchParams.append("count", count.toString());
			searchParams.append("skip", pageParam.toString());
			if (demo) searchParams.append("demo", "true");

			return fetch(
				`/api/activity${sourceId ? `/${sourceId}` : ""}?${searchParams.toString()}`,
			).then(handleFetchResponse("Error fetching activities"));
		},
		initialPageParam: 0,
		getNextPageParam: (lastPage, _allPages, lastPageParam) => {
			if (lastPage.length < count) {
				return undefined;
			} else {
				return lastPageParam + count;
			}
		},
	});
}

export function useRefreshRSS() {
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

export function useClearActivity() {
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
