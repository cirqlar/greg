import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { useMemo } from "react";

export const Route = createFileRoute("/_layout/roadmap/$roadmap_id")({
	component: Roadmap,
});

type TTabChange = {
	type: "tab_added" | "tab_removed";
	tab_id: string;
	tab_db_id: number;
	tab_name: string;
	tab_slug: string;
};

type TCardAdded = {
	type: "card_added";
	current_card_id: string;
	current_card_db_id: number;
	current_card_name: string;
	current_card_description: string;
	current_card_image_url?: string;
	current_card_slug: string;
};

type TCardRemoved = {
	type: "card_removed";
	previous_card_id: string;
	previous_card_db_id: number;
	previous_card_name: string;
	previous_card_description: string;
	previous_card_image_url?: string;
	previous_card_slug: string;
};

type TCardModified = {
	type: "card_modified";
	previous_card_id: string;
	previous_card_db_id: number;
	previous_card_name: string;
	previous_card_description: string;
	previous_card_image_url?: string;
	previous_card_slug: string;
	current_card_id: string;
	current_card_db_id: number;
	current_card_name: string;
	current_card_description: string;
	current_card_image_url?: string;
	current_card_slug: string;
};

type TRoadmapChange = {
	id: number;
} & (TTabChange | TCardAdded | TCardRemoved | TCardModified);

function Roadmap() {
	const { roadmap_id } = Route.useParams();

	const roadmapChanges = useQuery<TRoadmapChange[]>({
		queryKey: ["roadmap", roadmap_id],
		queryFn: () =>
			fetch(`/api/roadmap_activity/${roadmap_id}`).then(async (res) => {
				if (res.ok) {
					return res.json();
				} else {
					const err = await res.json();
					console.log("Error fetching changes", err);
					throw err;
				}
			}),
	});
	const roadmapChangesMapped = useMemo(() => (roadmapChanges.data ?? [])
		.map(c => {

			let current_card_description = null;
			let previous_card_description = null;
	
			if (c.type == "card_added" || c.type == "card_modified") {
				current_card_description = c.current_card_description.split("\n").filter(s => s.trim().length != 0).map(s => <p className="px-2 break-words">{s}</p>)
			}
			if (c.type == "card_removed" || c.type == "card_modified") {
				previous_card_description = c.previous_card_description.split("\n").filter(s => s.trim().length != 0).map(s => <p className="px-2 break-words">{s}</p>)
			}

			const changes = [];
			if (c.type == "card_modified") {
				if (c.current_card_name != c.previous_card_name) {
					changes.push("Title");
				}
				if (c.current_card_description != c.previous_card_description) {
					changes.push("Description");
				}
				if (c.current_card_image_url != c.previous_card_image_url) {
					changes.push("Image");
				}
			}

			return { ...c, current_card_description, previous_card_description, ...(c.type == "card_modified" ? { changes: changes.join(", ") } : {}) };
		})

	, [roadmapChanges.data])

	if (roadmapChanges.isError) {
		return (
			<div className="max-w-4xl mx-auto px-4">
				<h3 className="text-2xl font-bold mb-4">Changes</h3>

				<p>There's been an error fetching changes</p>
			</div>
		);
	}

	if (roadmapChanges.isPending) {
		return (
			<div className="max-w-4xl mx-auto px-4">
				<h3 className="text-2xl font-bold mb-4">Changes</h3>

				<p>Fetching changes...</p>
			</div>
		);
	}
	
	if (roadmapChanges.data.length === 0) {
		return (
			<div className="max-w-4xl mx-auto px-4">
				<h3 className="text-2xl font-bold mb-4">Changes</h3>

				<p>No changes for this roadmap</p>
			</div>
		);
	}

	return (
		<>
			<div className="max-w-4xl mx-auto px-4">
				<h3 className="text-2xl font-bold mb-4">Changes</h3>
				<div className="grid sm:grid-cols-2 grid-cols-1 gap-5">
					{roadmapChangesMapped.map(change => {
						if (change.type === "tab_added") {
							return <div className="col-span-full">Tab added: {change.tab_name} <a target="_blank" referrerPolicy="no-referrer" href={`${import.meta.env.VITE_ROADMAP_URL}tabs/${change.tab_slug}`}>link</a></div>;
						} else if (change.type === "tab_removed") {
							return <div className="col-span-full">Tab removed: {change.tab_name}</div>;
						} else if (change.type === "card_added") {
							return (
								<div className="flex flex-col border-2 rounded border-green-700 overflow-hidden pb-2 text-sm">
									<div className="w-full aspect-video mb-2">{change.current_card_image_url && <img className="w-full h-full object-cover" loading="lazy" src={change.current_card_image_url} />}</div>
									<h3 className="text-xl px-2 mb-2">{change.current_card_name} <a className="text-sm" target="_blank" referrerPolicy="no-referrer" href={`${import.meta.env.VITE_ROADMAP_URL}c/${change.current_card_slug}`}>link</a></h3>
									{change.current_card_description}
								</div>
							);
						} else if (change.type === "card_removed") {
							return (
								<div className="flex flex-col border-2 rounded border-red-700 overflow-hidden pb-2 text-sm">
									{/* Image resource is removed with card it seems */}
									{/* <div className="w-full aspect-video mb-2">{change.previous_card_image_url && <img className="w-full h-full object-cover" loading="lazy" src={change.previous_card_image_url} />}</div> */}
									<h3 className="text-xl pt-2 px-2 mb-2">{change.previous_card_name}</h3>
									{change.previous_card_description}
								</div>
							);
						} else if (change.type === "card_modified"){
							return (
								<div className="grid grid-cols-2 gap-4 col-span-full border-2 rounded border-blue-700 overflow-hidden pb-2 text-sm">
									<div className="w-full aspect-video">{change.previous_card_image_url && <img className="w-full h-full object-cover" loading="lazy" src={change.previous_card_image_url} />}</div>
									<div className="w-full aspect-video">{change.current_card_image_url && <img className="w-full h-full object-cover" loading="lazy" src={change.current_card_image_url} />}</div>
									<div className="col-span-full px-2">
										Changes: {change.changes}
									</div>
									<div><h3 className="text-xl px-2">{change.previous_card_name}</h3></div>
									<div><h3 className="text-xl px-2">{change.current_card_name} <a className="text-sm" target="_blank" referrerPolicy="no-referrer" href={`${import.meta.env.VITE_ROADMAP_URL}c/${change.current_card_slug}`}>link</a></h3></div>
									<div>{change.previous_card_description}</div>
									<div>{change.current_card_description}</div>
								</div>
							);
						}
					})}
				</div>
			</div>
			<div className="max-w-lg mx-auto px-4 mb-8 flex justify-end">
				{/* <button
					disabled={loading}
					className="h-full bg-green-700 px-4 py-3 uppercase font-bold rounded"
					onClick={async (e) => {
						e.preventDefault();

						setLoading(true);
						try {
							await recheck.mutateAsync();
						} catch {
							console.log("Recheck failed");
						}
						setLoading(false);
					}}
				>
					Recheck
				</button> */}
			</div>
			<div className="max-w-lg mx-auto px-4 mb-8">
				{/* <h3 className="text-2xl font-bold mb-4">Clear Activity</h3>
				<form
					onSubmit={async (e) => {
						e.preventDefault();

						setLoading(true);
						try {
							await clearActivities.mutateAsync(num);
						} catch {
							console.log("Recheck failed");
						}
						setLoading(false);
					}}
				>
					<label className="block mb-4 text-xl" htmlFor="password">
						Number to clear (empty clears all)
					</label>
					<div className="flex">
						<input
							value={num}
							disabled={loading}
							onChange={(e) => setNum(Number(e.target.value))}
							className="block w-full text-black mr-2 px-4 py-3 rounded"
							id="password"
							type="number"
							placeholder="Url"
						/>
						<button
							disabled={loading}
							className="h-full bg-green-700 px-4 py-3 uppercase font-bold rounded"
							type="submit"
						>
							Clear
						</button>
					</div>
				</form> */}
			</div>
			
		</>
	);
}
