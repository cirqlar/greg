import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import type { TSource } from "./types";
import { handleFetchResponse } from "./util";

export function useSources() {
	return useQuery<TSource[]>({
		queryKey: ["sources"],
		queryFn: () =>
			fetch("/api/sources").then(
				handleFetchResponse("Error fetching sources"),
			),
	});
}

export function useAddSource() {
	const queryClient = useQueryClient();

	return useMutation({
		mutationFn: (source: { url: string }) =>
			fetch("/api/source/new", {
				method: "POST",
				headers: {
					"Content-Type": "application/json",
				},
				body: JSON.stringify(source),
			}).then(handleFetchResponse("Error adding source")),
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ["sources"] });
		},
	});
}

export function useEnableSource() {
	const queryClient = useQueryClient();

	return useMutation({
		mutationFn: ({ id, enable }: { id: number; enable: boolean }) =>
			fetch(`/api/source/${id}/enable/${enable}`, {
				method: "POST",
				headers: {
					"Content-Type": "application/json",
				},
			}).then(handleFetchResponse("Error enabling source")),
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ["sources"] });
		},
	});
}

export function useDeleteSource() {
	const queryClient = useQueryClient();

	return useMutation({
		mutationFn: (id: number) =>
			fetch(`/api/source/${id}`, {
				method: "DELETE",
			}).then(handleFetchResponse("Error deleting source")),
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ["sources"] });
		},
	});
}
