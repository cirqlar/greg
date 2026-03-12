import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { handleFetchResponse } from "./util";

export function useLoginQuery() {
	return useQuery({
		queryKey: ["loggedin"],
		queryFn: (): Promise<boolean> =>
			fetch("/api/check-logged-in").then(
				handleFetchResponse("Error checking logged in"),
			),
		// refetchOnMount: false,
		refetchOnWindowFocus: false,
		refetchOnReconnect: false,
		retry: false,
	});
}

export function useLogoutMutation() {
	const queryClient = useQueryClient();

	return useMutation({
		mutationFn: () =>
			fetch("/api/logout", {
				method: "DELETE",
			}).then(handleFetchResponse("Error logging out")),
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ["loggedin"] });
		},
	});
}
