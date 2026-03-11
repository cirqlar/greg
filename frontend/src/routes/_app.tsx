import { useEffect, useEffectEvent, useState } from "react";
import {
	Outlet,
	createFileRoute,
	useNavigate,
	useRouterState,
} from "@tanstack/react-router";
import * as v from "valibot";
import { DocListIcon, PowerIcon, RSSIcon } from "@storybook/icons";

import { Button, Link } from "@/components/buttons";
import { useLoginQuery, useLogoutMutation } from "@/query/login";
import { updateProcessing } from "@/stores/processing";

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

	const loginQuery = useLoginQuery();
	const logoutMutation = useLogoutMutation();

	const [loggingOut, setLoggingOut] = useState(false);

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
			<header className="fixed top-6 right-4 z-50 flex items-center gap-4">
				<Link
					// Link
					to="/rss/{-$sourceId}"
					activeProps={{ className: "font-bold" }}
					search={(prev) => prev}
					// Custom
					Icon={RSSIcon}
					iconLabel="Link to RSS page"
				>
					RSS
				</Link>
				<Link
					// Link
					to="/roadmaps"
					activeOptions={{ exact: true }}
					activeProps={{ className: "font-bold" }}
					search={(prev) => prev}
					// Custom
					Icon={DocListIcon}
					iconLabel="Link to Roadmap Page"
				>
					Roadmap
				</Link>
				<Button
					Icon={PowerIcon}
					iconLabel="Logout"
					disabled={loggingOut}
					animate={loggingOut}
					theme="red"
					onClick={async () => {
						setLoggingOut(true);
						updateProcessing(true);
						if (!demo) {
							try {
								await logoutMutation.mutateAsync();
							} catch (e) {
								console.log("error logging out", e);
								// Kill logout attempt rather than navigate?
							}
						}
						setLoggingOut(false);
						updateProcessing(false);

						navigate({ to: "/", search: {} });
					}}
				>
					LogOut
				</Button>
			</header>
			<Outlet />
		</>
	);
}
