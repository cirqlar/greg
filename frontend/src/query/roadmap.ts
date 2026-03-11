import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import type {
	TRoadmapActivity,
	TRoadmapChange,
	TRTab,
	TWatchedTab,
} from "./types";
import { handleFetchResponse } from "./util";

export function useRoadmapActivity(demo?: boolean) {
	return useQuery<TRoadmapActivity[]>({
		queryKey: ["roadmap_activity", demo ?? false],
		queryFn: () =>
			fetch(`/api/roadmap_activity${demo ? "?demo=true" : ""}`).then(
				handleFetchResponse("Error fetching roadmap activity"),
			),
	});
}

export function useRoadmapTabs(demo?: boolean) {
	return useQuery<TRTab[]>({
		queryKey: ["most_recent_tabs", demo ?? false],
		queryFn: () =>
			fetch(`/api/most_recent_tabs${demo ? "?demo=true" : ""}`).then(
				handleFetchResponse("Error fetching most recent tabs"),
			),
	});
}

export function useRoadmapWatchedTabs(demo?: boolean) {
	return useQuery<TWatchedTab[]>({
		queryKey: ["watched_tabs", demo ?? false],
		queryFn: () =>
			fetch(`/api/watched_tabs${demo ? "?demo=true" : ""}`).then(
				handleFetchResponse("Error fetching watched tabs"),
			),
	});
}

export function useRoadmapChanges(roadmapId: number, demo?: boolean) {
	return useQuery<TRoadmapChange[]>({
		queryKey: ["roadmap", roadmapId, demo ?? false],
		queryFn: () =>
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
