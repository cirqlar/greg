import { useCallback, useState } from "react";
import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { ArrowRightIcon, EditorIcon } from "@storybook/icons";
import * as v from "valibot";

import { Button, Link } from "@/components/buttons";

export const Route = createFileRoute("/")({
	component: Index,
	validateSearch: v.object({
		redirect: v.optional(v.string()),
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
		async (e: React.SubmitEvent) => {
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
					navigate({ to: redirect || "/rss/{-$sourceId}" });
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
		navigate({ to: redirect || "/rss/{-$sourceId}" });
	}

	return (
		<div className="mx-auto flex h-full min-h-full w-full max-w-80 flex-col justify-center gap-4">
			<h1 className="mb-6 text-center text-6xl">GREG</h1>
			<div className="flex flex-col gap-2">
				<form onSubmit={submit}>
					<label
						className="sr-only mb-4 block text-2xl"
						htmlFor="password"
					>
						Password
					</label>
					<div className="relative flex">
						<input
							value={password}
							disabled={loading}
							onChange={(e) => setPassword(e.target.value)}
							className="block w-full rounded-full bg-white/20 py-3 pr-13 pl-4 text-white outline-none focus-visible:border-2 focus-visible:border-white focus-visible:py-2.5 focus-visible:pr-12.5 focus-visible:pl-3.5"
							id="password"
							type="password"
							placeholder="Password"
						/>
						<Button
							Icon={ArrowRightIcon}
							iconLabel="Log in"
							disabled={loading}
							animate={loading}
							error={!!error}
							type="submit"
							className="absolute top-0 right-0"
						/>
					</div>
				</form>
				{error && (
					<p className="text-center text-sm text-red-500">{error}</p>
				)}
			</div>
			<div className="flex items-center gap-2">
				<div className="h-0.5 flex-1 bg-white/20"></div>
				<p className="flex-none text-center">OR</p>
				<div className="h-0.5 flex-1 bg-white/20"></div>
			</div>
			<Link
				// Link
				to="/rss/{-$sourceId}"
				params={{ sourceId: undefined }}
				search={{ demo: true }}
				// Custom
				Icon={EditorIcon}
				iconLabel="Enter Demo"
			>
				Enter Demo
			</Link>
		</div>
	);
}
