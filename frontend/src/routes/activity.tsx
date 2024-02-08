import { createFileRoute } from "@tanstack/react-router";


export const Route = createFileRoute('/activity')({
	component: Activity,
})

function Activity() {
	return (
		<div></div>
	)
}