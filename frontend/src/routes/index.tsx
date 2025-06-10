import { useQuery, useQueryClient } from "@tanstack/react-query";
import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useCallback, useState } from "react";

export const Route = createFileRoute("/")({
	component: Index,
	validateSearch: (search) => ({
		redirect: typeof search.redirect === "string" ? search.redirect : "",
	}),
});

function Index() {
	const navigate = useNavigate();
	const queryClient = useQueryClient();
	const { redirect } = Route.useSearch();

	const loginQuery = useQuery<boolean>({
		queryKey: ["loggedin"],
		queryFn: () => fetch("/api/check-logged-in").then((res) => res.json()),
		// refetchOnMount: false,
		refetchOnWindowFocus: false,
		refetchOnReconnect: false,
	});

	const [password, setPassword] = useState("");
	const [loading, setLoading] = useState(false);
	const [error, setError] = useState("");

	const submit = useCallback(
		async (e: React.FormEvent) => {
			e.preventDefault();
			setLoading(true);

			try {
				const res = await fetch("/api/login", {
					method: "POST",
					headers: { "Content-Type": "application/json" },
					body: JSON.stringify({ password }),
				});
				if (res.ok) {
					queryClient.invalidateQueries({ queryKey: ["loggedin"] });
					navigate({ to: redirect || "/activity" });
				} else {
					const error = await res.json();
					throw error;
				}
				// eslint-disable-next-line @typescript-eslint/no-explicit-any
			} catch (error: any) {
				setError(error?.message || "Issue logging in");
			}

			setLoading(false);
		},
		[navigate, password, queryClient, redirect],
	);

	if (loginQuery.isSuccess && loginQuery.data) {
		navigate({ to: redirect || "/activity" });
	}

	return (
		<div className="w-full min-h-full h-full flex items-center justify-center">
			{loginQuery.isPending ? (
				<p>Loading...</p>
			) : (
				<div className="max-w-80 w-full">
					<form onSubmit={submit}>
						<label
							className="block mb-4 text-2xl"
							htmlFor="password"
						>
							Password
						</label>
						<div className="flex">
							<input
								value={password}
								disabled={loading}
								onChange={(e) => setPassword(e.target.value)}
								className="block w-full text-black mr-2 px-4 py-3 rounded"
								id="password"
								type="password"
								placeholder="Password"
							/>
							<button
								disabled={loading}
								className="h-full bg-green-700 px-4 py-3 uppercase font-bold rounded"
								type="submit"
							>
								Login
							</button>
						</div>
					</form>
					{error && <p>{error}</p>}
				</div>
			)}
		</div>
	);
}
