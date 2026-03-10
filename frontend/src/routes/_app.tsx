import { useEffect, useEffectEvent } from "react";
import {
	Outlet,
	createFileRoute,
	useNavigate,
	useRouterState,
} from "@tanstack/react-router";
import { useQuery } from "@tanstack/react-query";
import * as v from "valibot";
import { DocListIcon, RSSIcon } from "@storybook/icons";

import { Link } from "@/components/buttons";

export const Route = createFileRoute("/_app")({
	component: RouteComponent,
	validateSearch: v.object({
		demo: v.optional(
			v.pipe(
				v.any(),
				v.transform(() => true),
			),
		),
	}),
});

function RouteComponent() {
	const navigate = useNavigate();
	const routerState = useRouterState();
	const { demo } = Route.useSearch();

	const loginQuery = useQuery<boolean>({
		queryKey: ["loggedin"],
		queryFn: () => fetch("/api/check-logged-in").then((res) => res.json()),
		// refetchOnMount: false,
		refetchOnWindowFocus: false,
		refetchOnReconnect: false,
		retry: false,
	});

	const pathname = useEffectEvent(() => routerState.location.pathname);

	useEffect(() => {
		console.log(pathname());

		if (!loginQuery.isFetching && !loginQuery.data && !demo) {
			navigate({
				to: "/",
				search: {
					redirect: pathname(),
				},
			});
		}
	}, [loginQuery.data, loginQuery.isFetching, demo, navigate]);

	return (
		<>
			<header className="fixed top-6 right-6 flex items-center gap-4">
				<Link
					to="/rss/{-$sourceId}"
					Icon={RSSIcon}
					iconLabel="Link to RSS page"
					activeProps={{ className: "font-bold" }}
				>
					RSS
				</Link>
				<Link
					to="/roadmaps"
					Icon={DocListIcon}
					iconLabel="Link to Roadmap Page"
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
