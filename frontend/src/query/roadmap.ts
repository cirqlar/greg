import {
	useInfiniteQuery,
	useMutation,
	useQuery,
	useQueryClient,
} from "@tanstack/react-query";

import type {
	TRoadmapActivity,
	TRoadmapChange,
	TRTab,
	TWatchedTab,
} from "./types";
import { handleFetchResponse } from "./util";

export function useRoadmapActivity(demo?: boolean) {
	return useQuery({
		queryKey: ["roadmap_activity", demo ?? false],
		queryFn: (): Promise<TRoadmapActivity[]> =>
			fetch(`/api/roadmap_activity${demo ? "?demo=true" : ""}`).then(
				handleFetchResponse("Error fetching roadmap activity"),
			),
	});
}

export function useInfiniteRoadmapActivity(demo?: boolean, count: number = 35) {
	return useInfiniteQuery({
		queryKey: ["roadmap_activity", count, demo ?? false],
		queryFn: ({ pageParam }): Promise<TRoadmapActivity[]> => {
			const searchParams = new URLSearchParams();
			searchParams.append("count", count.toString());
			searchParams.append("skip", pageParam.toString());
			if (demo) searchParams.append("demo", "true");

			return fetch(
				`/api/roadmap_activity?${searchParams.toString()}`,
			).then(handleFetchResponse("Error fetching roadmap activity"));
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

export function useRoadmapTabs(demo?: boolean) {
	return useQuery({
		queryKey: ["most_recent_tabs", demo ?? false],
		queryFn: (): Promise<TRTab[]> =>
			fetch(`/api/most_recent_tabs${demo ? "?demo=true" : ""}`).then(
				handleFetchResponse("Error fetching most recent tabs"),
			),
	});
}

export function useRoadmapWatchedTabs(demo?: boolean) {
	return useQuery({
		queryKey: ["watched_tabs", demo ?? false],
		queryFn: (): Promise<TWatchedTab[]> =>
			fetch(`/api/watched_tabs${demo ? "?demo=true" : ""}`).then(
				handleFetchResponse("Error fetching watched tabs"),
			),
	});
}

export function useRoadmapChanges(roadmapId: number, demo?: boolean) {
	return useQuery({
		queryKey: ["roadmap", roadmapId, demo ?? false],
		queryFn: (): Promise<TRoadmapChange[]> =>
			fetch(
				`/api/roadmap_activity/${roadmapId}${demo ? "?demo=true" : ""}`,
			).then(handleFetchResponse("Error fetching changes")),
	});
}

export function useUnwatchTabMutation() {
	const queryClient = useQueryClient();

	return useMutation({
		mutationFn: (id: number) =>
			fetch(`/api/watched_tabs/${id}`, {
				method: "DELETE",
			}).then(handleFetchResponse("Error unwatching tab")),
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ["watched_tabs"] });
		},
	});
}

export function useWatchTabMutation() {
	const queryClient = useQueryClient();

	return useMutation({
		mutationFn: (tab_id: string) =>
			fetch(`/api/watched_tabs/add/${tab_id}`, {
				method: "POST",
			}).then(handleFetchResponse("Error watching tab")),
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ["watched_tabs"] });
		},
	});
}

export function useRefreshRoadmap() {
	const queryClient = useQueryClient();

	return useMutation({
		mutationFn: () =>
			fetch("/api/recheck_roadmap", {
				method: "POST",
			}).then(handleFetchResponse("Error refreshing roadmap")),
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ["roadmap_activity"] });
			queryClient.invalidateQueries({ queryKey: ["most_recent_tabs"] });
			queryClient.invalidateQueries({ queryKey: ["watched_tabs"] });
		},
	});
}
