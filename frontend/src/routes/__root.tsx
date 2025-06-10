import { createRootRoute, Outlet } from "@tanstack/react-router";
// import { TanStackRouterDevtools } from '@tanstack/router-devtools'

export const Route = createRootRoute({
	component: () => (
		<div className="w-full min-h-full h-full dark:bg-black bg-white dark:text-white text-black overflow-auto">
			<>
				<Outlet />
				{/* <TanStackRouterDevtools /> */}
			</>
		</div>
	),
});
