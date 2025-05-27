import { Link, Outlet, createFileRoute } from "@tanstack/react-router"


export const Route = createFileRoute('/_layout')({
	component: Layout,
})

function Layout() {
	return (
		<>
			<header className="h-16 bg-black text-white flex items-center px-6">
				<Link className="mr-4 hover:text-green-500 focus:text-green-500" to="/sources" activeOptions={{ exact: true }} activeProps={{ className: 'font-bold' }} >Sources</Link>
				<Link className="mr-4 hover:text-green-500 focus:text-green-500" to="/activity" activeOptions={{ exact: true }} activeProps={{ className: 'font-bold' }} >Activity</Link>
				<Link className="hover:text-green-500 focus:text-green-500" to="/roadmaps" activeOptions={{ exact: true }} activeProps={{ className: 'font-bold' }} >Roadmap</Link>
			</header>
			<Outlet />
		</>
	)
}