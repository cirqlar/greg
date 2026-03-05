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

type TSingleTextChange = { text: string; wrap: boolean };

function render_change_paragraph(
	change_text: TSingleTextChange[],
	disambig: string,
	green: boolean,
) {
	return (
		<p
			key={disambig}
			className={`px-2 wrap-break-word ${change_text[0].text[0] === "-" || change_text[0].text[0] === "." ? "pl-6" : ""}`}
		>
			{change_text.map(({ text, wrap }, k) => (
				<Fragment key={`${text} + ${disambig} + ${k}`}>
					{wrap ? (
						<span
							className={
								green ? "text-green-400" : "text-red-400"
							}
						>
							{text}
						</span>
					) : (
						text
					)}
				</Fragment>
			))}
		</p>
	);
}

function render_paragraphs(text: string) {
	return text
		.split("\n")
		.filter((s) => s.trim().length != 0)
		.map((s) => (
			<p
				key={s}
				className={`px-2 wrap-break-word ${s[0] === "-" || s[0] === "." ? "pl-6" : ""}`}
			>
				{s}
			</p>
		));
}

function TabChange({ change }: { change: { id: number } & TTabChange }) {
	return (
		<div key={change.id} className="col-span-full">
			Tab {change.type === "tab_added" ? "added" : "removed"}:{" "}
			{change.tab_name}{" "}
			{change.type === "tab_added" && (
				<a
					target="_blank"
					referrerPolicy="no-referrer"
					href={`${import.meta.env.VITE_ROADMAP_URL}/tabs/${change.tab_slug}`}
				>
					link
				</a>
			)}
		</div>
	);
}

function CardChange({
	change,
}: {
	change: { id: number } & (TCardAdded | TCardRemoved);
}) {
	const description = render_paragraphs(
		change.type === "card_added"
			? change.current_card_description
			: change.previous_card_description,
	);

	return (
		<div
			key={change.id}
			className={`flex flex-col overflow-hidden rounded-sm border-2 pb-2 text-sm ${change.type === "card_added" ? "border-green-700" : "border-red-700"}`}
		>
			{change.type === "card_added" && (
				<div className="mb-2 aspect-video w-full">
					<img
						className="h-full w-full object-cover"
						loading="lazy"
						src={change.current_card_image_url}
					/>
				</div>
			)}
			<h3 className="mb-2 px-2 text-xl">
				{change.type === "card_added"
					? change.current_card_name
					: change.previous_card_name}{" "}
				{change.type === "card_added" && (
					<a
						className="text-sm"
						target="_blank"
						referrerPolicy="no-referrer"
						href={`${import.meta.env.VITE_ROADMAP_URL}/c/${change.current_card_slug}`}
					>
						link
					</a>
				)}
			</h3>
			<p className="mb-2 px-2">{change.card_tab_name}</p>
			{description}
		</div>
	);
}

function CardMod({ change }: { change: { id: number } & TCardModified }) {
	const desc_changed =
		change.current_card_description != change.previous_card_description;
	const img_changed =
		change.current_card_image_url != change.previous_card_image_url;

	const title =
		change.current_card_name != change.previous_card_name
			? diffWordsWithSpace(
					change.previous_card_name,
					change.current_card_name,
				).map((change) => {
					if (change.added) {
						return (
							<span key={change.value} className="text-green-400">
								{change.value}
							</span>
						);
					} else if (change.removed) {
						return (
							<span key={change.value} className="text-red-400">
								{change.value}
							</span>
						);
					} else {
						return <span key={change.value}>{change.value}</span>;
					}
				})
			: change.current_card_name;

	const { previous_card_description, current_card_description } =
		useMemo(() => {
			if (
				change.previous_card_description ==
				change.current_card_description
			) {
				return {
					previous_card_description: change.previous_card_description,
					current_card_description: change.current_card_description,
				};
			} else {
				const diffs = diffWordsWithSpace(
					change.previous_card_description,
					change.current_card_description,
				);

				let previous_card_description: ReactNode[] = [];
				let current_card_description: ReactNode[] = [];

				let previous_text: TSingleTextChange[] = [];
				let current_text: TSingleTextChange[] = [];

				for (let i = 0; i < diffs.length; i++) {
					let diff = diffs[i];

					if (diff.value.startsWith("\n")) {
						if (!diff.added && previous_text.length > 0) {
							previous_card_description.push(
								render_change_paragraph(
									previous_text,
									`${change.current_card_id} + ${i}`,
									false,
								),
							);

							previous_text = [];
						}

						if (!diff.removed && current_text.length > 0) {
							current_card_description.push(
								render_change_paragraph(
									current_text,
									`${change.current_card_id} + ${i}`,
									true,
								),
							);

							current_text = [];
						}
					}

					let arr = diff.value.split("\n");

					for (let j = 0; j < arr.length; j++) {
						const is_last = j === arr.length - 1;
						const current_v = arr[j];

						if (current_v.length === 0) continue;

						if (!diff.added) {
							previous_text.push({
								text: current_v,
								wrap: diff.removed,
							});
						}

						if (!diff.removed) {
							current_text.push({
								text: current_v,
								wrap: diff.added,
							});
						}

						if (!is_last) {
							if (!diff.added && previous_text.length > 0) {
								previous_card_description.push(
									render_change_paragraph(
										previous_text,
										`${change.current_card_id} + ${i} + ${j}`,
										false,
									),
								);

								previous_text = [];
							}

							if (!diff.removed && current_text.length > 0) {
								current_card_description.push(
									render_change_paragraph(
										current_text,
										`${change.current_card_id} + ${i} + ${j}`,
										true,
									),
								);

								current_text = [];
							}
						}
					}
				}

				if (previous_text.length > 0) {
					previous_card_description.push(
						render_change_paragraph(
							previous_text,
							change.current_card_id,
							false,
						),
					);

					previous_text = [];
				}

				if (current_text.length > 0) {
					current_card_description.push(
						render_change_paragraph(
							current_text,
							change.current_card_id,
							true,
						),
					);

					current_text = [];
				}
				return { previous_card_description, current_card_description };
			}
		}, [
			change.current_card_description,
			change.previous_card_description,
			change.current_card_id,
		]);

	return (
		<div
			key={change.id}
			className={`grid gap-4 overflow-hidden rounded-sm border-2 border-blue-700 pb-2 text-sm ${desc_changed ? "col-span-full grid-cols-2" : ""}`}
		>
			<div
				className={`col-span-full aspect-video w-full ${img_changed ? "rounded-t border-2 border-green-400" : ""}`}
			>
				{change.current_card_image_url && (
					<img
						className="h-full w-full object-cover"
						loading="lazy"
						src={change.current_card_image_url}
					/>
				)}
			</div>
			<div className="col-span-full px-2">
				<p>Tab: {change.card_tab_name}</p>
			</div>

			<div className="col-span-full">
				<h3 className="px-2 text-xl">
					{title}{" "}
					<a
						className="text-sm"
						target="_blank"
						referrerPolicy="no-referrer"
						href={`${import.meta.env.VITE_ROADMAP_URL}/c/${change.current_card_slug}`}
					>
						link
					</a>
				</h3>
			</div>
			{desc_changed && <div>{previous_card_description}</div>}
			<div>{current_card_description}</div>
		</div>
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

	return (
		<>
			<div className="mx-auto max-w-4xl px-4">
				<h3 className="mb-4 text-2xl font-bold">Changes</h3>
				{roadmapChanges.isError ? (
					<p>There's been an error fetching changes</p>
				) : roadmapChanges.isPending ? (
					<p>Fetching changes...</p>
				) : roadmapChanges.data.length == 0 ? (
					<p>No changes for this roadmap</p>
				) : (
					<div className="grid grid-cols-1 gap-5 sm:grid-cols-2">
						{roadmapChanges.data.map((change) => {
							if (
								change.type === "tab_added" ||
								change.type === "tab_removed"
							) {
								return (
									<TabChange
										change={change}
										key={change.id}
									/>
								);
							} else if (
								change.type === "card_added" ||
								change.type === "card_removed"
							) {
								return (
									<CardChange
										change={change}
										key={change.id}
									/>
								);
							} else if (change.type === "card_modified") {
								return (
									<CardMod change={change} key={change.id} />
								);
							}
						})}
					</div>
				)}
			</div>
		</>
	);
}
