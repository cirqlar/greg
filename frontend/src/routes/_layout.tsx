import { useQuery } from "@tanstack/react-query";
import {
	Link,
	Outlet,
	createFileRoute,
	useNavigate,
	useRouterState,
} from "@tanstack/react-router";
import { useEffect } from "react";

export const Route = createFileRoute("/_layout")({
	component: Layout,
});

function Layout() {
	const navigate = useNavigate();
	const routerState = useRouterState();

	const loginQuery = useQuery<boolean>({
		queryKey: ["loggedin"],
		queryFn: () => fetch("/api/check-logged-in").then((res) => res.json()),
		// refetchOnMount: false,
		refetchOnWindowFocus: false,
		refetchOnReconnect: false,
		retry: false,
	});

	useEffect(() => {
		console.log(routerState.location.pathname);
		if (!loginQuery.isFetching && !loginQuery.data) {
			navigate({
				to: "/",
				search: {
					redirect: routerState.location.pathname,
				},
			});
		}
		// Having routerState.location.pathname as a dependency makes the effect run
		// again after navigation which overwrites the search param with "/"
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, [loginQuery.data, loginQuery.isFetching, navigate]);

	return (
		<>
			<header className="h-16 bg-black text-white flex items-center px-6">
				<Link
					className="mr-4 hover:text-green-500 focus:text-green-500"
					to="/sources"
					activeOptions={{ exact: true }}
					activeProps={{ className: "font-bold" }}
				>
					Sources
				</Link>
				<Link
					className="mr-4 hover:text-green-500 focus:text-green-500"
					to="/activity"
					activeOptions={{ exact: true }}
					activeProps={{ className: "font-bold" }}
				>
					Activity
				</Link>
				<Link
					className="hover:text-green-500 focus:text-green-500"
					to="/roadmaps"
					activeOptions={{ exact: true }}
					activeProps={{ className: "font-bold" }}
				>
					Roadmap
				</Link>
			</header>
			<Outlet />
		</>
	);
}
