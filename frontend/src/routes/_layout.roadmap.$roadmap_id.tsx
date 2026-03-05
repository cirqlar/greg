import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { diffWordsWithSpace } from "diff";
import { useMemo, ReactNode, Fragment } from "react";

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
	card_tab_name: string;
};

type TCardRemoved = {
	type: "card_removed";
	previous_card_id: string;
	previous_card_db_id: number;
	previous_card_name: string;
	previous_card_description: string;
	previous_card_image_url?: string;
	previous_card_slug: string;
	card_tab_name: string;
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
	card_tab_name: string;
};

type TRoadmapChange = {
	id: number;
} & (TTabChange | TCardAdded | TCardRemoved | TCardModified);


function render_change_paragraph(change_text: { text: string, wrap: boolean }[], disambig: string, green: boolean) {
	return (
		<p key={disambig} className={`px-2 wrap-break-word ${change_text[0].text[0] === '-' || change_text[0].text[0] === '.' ? 'pl-6' : ''}`}>
			{change_text.map(({ text, wrap }, k) => (
				<Fragment key={`${text} + ${disambig} + ${k}`}>
					{wrap ? <span className={green ? "text-green-400" : "text-red-400"}>{text}</span> : text}
				</Fragment>
			))}
		</p>
	);
}

function Roadmap() {
	const { roadmap_id } = Route.useParams();

	const roadmapChanges = useQuery<TRoadmapChange[]>({
		queryKey: ["roadmap", roadmap_id],
		queryFn: () =>
			fetch(`/api/roadmap_activity/${roadmap_id}`).then(async (res) => {
				if (res.ok) {
					return res.json();
				} else {
					let err;
				try {
					err = await res.json();
				} catch {
					err = "Non-json return from response";
				}
					console.log("Error fetching changes", err);
					throw err;
				}
			}),
	});
	const roadmapChangesMapped = useMemo(() => (roadmapChanges.data ?? [])
		.map(c => {

			let current_card_description: ReactNode[] | null = null;
			let previous_card_description: ReactNode[] | null = null;

			if (c.type === "card_added") {
				current_card_description = c.current_card_description.split("\n").filter(s => s.trim().length != 0).map(s => <p key={s} className={`px-2 wrap-break-word ${s[0] === '-' || s[0] === '.' ? 'pl-6' : ''}`}>{s}</p>)
			} else if (c.type === "card_removed") {
				previous_card_description = c.previous_card_description.split("\n").filter(s => s.trim().length != 0).map(s => <p key={s} className={`px-2 wrap-break-word ${s[0] === '-' || s[0] === '.' ? 'pl-6' : ''}`}>{s}</p>)
			} else if (c.type === "card_modified") {
				if (c.previous_card_description !== c.current_card_description) {
					const changes = diffWordsWithSpace(c.previous_card_description, c.current_card_description);

					previous_card_description = [];
					current_card_description = [];

					let previous_text: { text: string, wrap: boolean }[] = [];
					let current_text: { text: string, wrap: boolean }[] = [];

					for (let i = 0; i < changes.length; i++) {
						let change = changes[i];

						if (change.value.startsWith('\n')) {
							if (!change.added && previous_text.length > 0) {
								previous_card_description.push(render_change_paragraph(previous_text, `${c.current_card_id} + ${i}`, false));

								previous_text = [];
							}

							if (!change.removed && current_text.length > 0) {
								current_card_description.push(render_change_paragraph(current_text, `${c.current_card_id} + ${i}`, true));

								current_text = []
							}
						}
						
						let arr = change.value.split('\n');
						
						for (let j = 0; j < arr.length; j++) {
							const is_last = j === arr.length - 1;
							const current_v = arr[j];

							if (current_v.length === 0) continue;

							if (!change.added) {
								previous_text.push({ text: current_v, wrap: change.removed });
							}

							if (!change.removed) {
								current_text.push({ text: current_v, wrap: change.added });
							}

							if (!is_last) {
								if (!change.added && previous_text.length > 0) {
									previous_card_description.push(render_change_paragraph(previous_text, `${c.current_card_id} + ${i} + ${j}`, false));

									previous_text = [];
								}

								if (!change.removed && current_text.length > 0) {
									current_card_description.push(render_change_paragraph(current_text, `${c.current_card_id} + ${i} + ${j}`, true));

									current_text = []
								}
							}
						}
					}

					if ( previous_text.length > 0) {
						previous_card_description.push(render_change_paragraph(previous_text, c.current_card_id, false));

						previous_text = [];
					}

					if (current_text.length > 0) {
						current_card_description.push(render_change_paragraph(current_text, c.current_card_id, true));

						current_text = []
					}
				} else {
					previous_card_description = c.previous_card_description.split("\n").filter(s => s.trim().length != 0).map(s => <p key={s} className={`px-2 wrap-break-word ${s[0] === '-' || s[0] === '.' ? 'pl-6' : ''}`}>{s}</p>)
					current_card_description = c.current_card_description.split("\n").filter(s => s.trim().length != 0).map(s => <p key={s} className={`px-2 wrap-break-word ${s[0] === '-' || s[0] === '.' ? 'pl-6' : ''}`}>{s}</p>)
				}
			}

			return { 
				...c,
				current_card_description,
				previous_card_description,
				...(c.type == "card_modified" 
					? {
						name_changed: c.current_card_name != c.previous_card_name,
						desc_changed: c.current_card_description != c.previous_card_description,
						img_changed: c.current_card_image_url != c.previous_card_image_url,
					} : {}
				)
			};
		})

	, [roadmapChanges.data])

	return (
		<>
			<div className="max-w-4xl mx-auto px-4">
				<h3 className="text-2xl font-bold mb-4">Changes</h3>
				{roadmapChanges.isError ?
					<p>There's been an error fetching changes</p>
				: roadmapChanges.isPending ?
					<p>Fetching changes...</p>
				: roadmapChanges.data.length == 0 ?
					<p>No changes for this roadmap</p>
				: (
					<div className="grid sm:grid-cols-2 grid-cols-1 gap-5">
						{roadmapChangesMapped.map(change => {
							if (change.type === "tab_added") {
								return <div key={change.id} className="col-span-full">Tab added: {change.tab_name} <a target="_blank" referrerPolicy="no-referrer" href={`${import.meta.env.VITE_ROADMAP_URL}/tabs/${change.tab_slug}`}>link</a></div>;
							} else if (change.type === "tab_removed") {
								return <div key={change.id} className="col-span-full">Tab removed: {change.tab_name}</div>;
							} else if (change.type === "card_added") {
								return (
									<div key={change.id} className="flex flex-col border-2 rounded-sm border-green-700 overflow-hidden pb-2 text-sm">
										<div className="w-full aspect-video mb-2">{change.current_card_image_url && <img className="w-full h-full object-cover" loading="lazy" src={change.current_card_image_url} />}</div>
										<h3 className="text-xl px-2 mb-2">{change.current_card_name} <a className="text-sm" target="_blank" referrerPolicy="no-referrer" href={`${import.meta.env.VITE_ROADMAP_URL}/c/${change.current_card_slug}`}>link</a></h3>
										<p className="px-2 mb-2">{change.card_tab_name}</p>
										{change.current_card_description}
									</div>
								);
							} else if (change.type === "card_removed") {
								return (
									<div key={change.id} className="flex flex-col border-2 rounded-sm border-red-700 overflow-hidden pb-2 text-sm">
										{/* Image resource is removed with card it seems */}
										{/* <div className="w-full aspect-video mb-2">{change.previous_card_image_url && <img className="w-full h-full object-cover" loading="lazy" src={change.previous_card_image_url} />}</div> */}
										<h3 className="text-xl pt-2 px-2 mb-2">{change.previous_card_name}</h3>
										<p className="px-2 mb-2">{change.card_tab_name}</p>
										{change.previous_card_description}
									</div>
								);
							} else if (change.type === "card_modified"){
								return (
									<div key={change.id} className={`grid gap-4 border-2 rounded-sm border-blue-700 overflow-hidden pb-2 text-sm ${change.desc_changed ? 'grid-cols-2 col-span-full' : ''}`}>
										<div className={`w-full aspect-video col-span-full ${change.img_changed ? 'border-2 border-green-400 rounded-t': ''}`}>{change.current_card_image_url && <img className="w-full h-full object-cover" loading="lazy" src={change.current_card_image_url} />}</div>
										<div className="col-span-full px-2">
											<p>Tab: {change.card_tab_name}</p>
										</div>
										
										<div className="col-span-full">
											<h3 className="text-xl px-2">
												{change.name_changed 
													? diffWordsWithSpace(change.previous_card_name, change.current_card_name).map(change => {
														if (change.added) {
															return <span key={change.value} className="text-green-400">{change.value}</span>
														} else if (change.removed) {
															return <span key={change.value} className="text-red-400">{change.value}</span>
														} else {
															return <span key={change.value}>{change.value}</span>
														}
													})
													: change.current_card_name
												} <a className="text-sm" target="_blank" referrerPolicy="no-referrer" href={`${import.meta.env.VITE_ROADMAP_URL}/c/${change.current_card_slug}`}>link</a>
											</h3>
										</div>
										{change.desc_changed && <div>{change.previous_card_description}</div>}
										<div>{change.current_card_description}</div>
									</div>
								);
							}
						})}
					</div>
				)}
			</div>
		</>
	);
}
