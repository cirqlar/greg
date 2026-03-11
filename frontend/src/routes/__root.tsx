import { createRootRoute, Outlet } from "@tanstack/react-router";
// import { TanStackRouterDevtools } from '@tanstack/router-devtools'

export const Route = createRootRoute({
	component: () => (
		<div className="h-full min-h-full w-full overflow-auto bg-white text-black dark:bg-black dark:text-white">
			<>
				<Outlet />
				{/* <TanStackRouterDevtools /> */}
			</>
		</div>
	),
});
