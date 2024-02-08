import { createFileRoute } from "@tanstack/react-router";


export const Route = createFileRoute('/_layout/activity')({
	component: Activity,
})

function Activity() {
	return (
		<div></div>
	)
}