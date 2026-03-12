import { createFileRoute } from "@tanstack/react-router";
import * as v from "valibot";

import { CardChange, CardMod, TabChange } from "@/components/roadmap";
import { useRoadmapChanges } from "@/query/roadmap";

export const Route = createFileRoute("/_app/roadmap_/$roadmapId")({
	component: RouteComponent,
	params: {
		parse: (rawParams) =>
			v.parse(
				v.object({
					roadmapId: v.pipe(
						v.string(),
						v.trim(),
						v.nonEmpty(),
						v.toNumber(),
					),
				}),
				rawParams,
			),
	},
});

function ChangeGrid() {
	const { roadmapId } = Route.useParams();
	const { demo } = Route.useSearch();

	const {
		data: roadmapChanges,
		error,
		isLoading,
	} = useRoadmapChanges(roadmapId, demo);

	if (isLoading) {
		return (
			<div className="">
				<p>Loading</p>
			</div>
		);
	}

	if (error || !roadmapChanges) {
		return (
			<div className="">
				<p>Error loading changes</p>
			</div>
		);
	}

	if (roadmapChanges.length === 0) {
		return (
			<div className="">
				<p>No changes</p>
			</div>
		);
	}

	return (
		<div className="max-w-4xl">
			<h3 className="mb-4 text-2xl font-bold">Changes</h3>
			<div className="grid grid-flow-row-dense grid-cols-1 gap-5 sm:grid-cols-2">
				{roadmapChanges.map((change) => {
					if (
						change.type === "tab_added" ||
						change.type === "tab_removed"
					) {
						return <TabChange change={change} key={change.id} />;
					} else if (
						change.type === "card_added" ||
						change.type === "card_removed"
					) {
						return <CardChange change={change} key={change.id} />;
					} else if (change.type === "card_modified") {
						return <CardMod change={change} key={change.id} />;
					}
				})}
			</div>
		</div>
	);
}

function RouteComponent() {
	return (
		<div className="relative flex justify-center px-4 pt-24 pb-8">
			<ChangeGrid />
		</div>
	);
}
