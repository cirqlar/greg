import { useMemo, type ReactNode, Fragment } from "react";

import clsx from "clsx";
import { diffWordsWithSpace } from "diff";
import { LinkIcon } from "@storybook/icons";

import { ExternalLink } from "@/components/buttons";
import type {
	TCardAdded,
	TCardModified,
	TCardRemoved,
	TTabChange,
} from "@/query/types";

type TSingleTextChange = { text: string; wrap: boolean };

function render_change_paragraph(
	change_text: TSingleTextChange[],
	disambig: string,
	green: boolean,
) {
	return (
		<p
			key={disambig}
			className={clsx(
				"wrap-break-word not-last:not-empty:mb-2",
				(change_text[0].text.startsWith("\t-") ||
					change_text[0].text.startsWith("\t.")) &&
					"pl-4",
			)}
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
				className={clsx(
					`px-5 wrap-break-word`,
					(s.startsWith("\t-") || s.startsWith("\t.")) && "pl-9",
				)}
			>
				{s}
			</p>
		));
}

function TopBar({
	absolute,
	changeColor,
	changeText,
	tabName,
	link,
}: {
	absolute: boolean;
	changeColor: string;
	changeText: string;
	tabName: string;
	link?: string;
}) {
	return (
		<div
			className={clsx(
				"flex items-start justify-between gap-2 px-5 pt-5",
				absolute && "absolute inset-x-0 top-0",
			)}
		>
			<div className="flex flex-wrap items-center gap-2">
				<div
					className={clsx(
						"flex items-center rounded-full px-3.5 py-1 text-sm text-black",
						changeColor,
					)}
				>
					<p>{changeText}</p>
				</div>

				<div className="flex items-center rounded-full bg-black px-3.5 py-1 text-sm text-white">
					<p>{tabName}</p>
				</div>
			</div>

			{link && (
				<ExternalLink
					Icon={LinkIcon}
					iconLabel="Open Card"
					href={link}
					size="small"
				/>
			)}
		</div>
	);
}

export function TabChange({ change }: { change: { id: number } & TTabChange }) {
	return (
		<div className="flex min-w-72 flex-col items-stretch gap-4 rounded-lg bg-white/20 px-5 py-3">
			<div className="flex items-center justify-between gap-2">
				<p
					className="flex-1 overflow-hidden text-lg text-nowrap text-ellipsis"
					title={change.tab_name}
				>
					{change.tab_name}
				</p>
			</div>

			<div className="flex items-stretch justify-between">
				<div
					className={clsx(
						"flex items-center rounded-full px-3.5 py-1 text-sm text-black",
						change.type === "tab_added"
							? "bg-green-400"
							: "bg-red-400",
					)}
				>
					<p>{change.type === "tab_added" ? "Added" : "Removed"}</p>
				</div>
				<div className="flex gap-2">
					<ExternalLink
						Icon={LinkIcon}
						iconLabel="Open Tab"
						href={`${import.meta.env.VITE_ROADMAP_URL}/tabs/${change.tab_slug}`}
						size="small"
					/>
				</div>
			</div>
		</div>
	);
}

export function CardChange({
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
			className="relative flex flex-col gap-4 overflow-hidden rounded-lg bg-white/20 pb-5 text-sm"
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

			<TopBar
				absolute={change.type === "card_added"}
				changeColor={clsx(
					change.type === "card_added"
						? "bg-green-400"
						: "bg-red-400",
				)}
				changeText={change.type === "card_added" ? "Added" : "Removed"}
				tabName={change.card_tab_name}
				link={
					change.type === "card_added"
						? `${import.meta.env.VITE_ROADMAP_URL}/c/${change.current_card_slug}`
						: undefined
				}
			/>

			<h3 className="px-5 text-xl">
				{change.type === "card_added"
					? change.current_card_name
					: change.previous_card_name}{" "}
			</h3>
			{description}
		</div>
	);
}

export function CardMod({
	change,
}: {
	change: { id: number } & TCardModified;
}) {
	const desc_changed =
		change.current_card_description != change.previous_card_description;
	const img_changed =
		change.current_card_image_url != change.previous_card_image_url;

	// diffWordsWithSpace(
	//     change.previous_card_name,
	//     change.current_card_name,
	// ).map((change) => {
	//     if (change.added) {
	//         return (
	//             <span key={change.value} className="text-green-400">
	//                 {change.value}
	//             </span>
	//         );
	//     } else if (change.removed) {
	//         return (
	//             <span key={change.value} className="text-red-400">
	//                 {change.value}
	//             </span>
	//         );
	//     } else {
	//         return <span key={change.value}>{change.value}</span>;
	//     }
	// })

	const title =
		change.current_card_name != change.previous_card_name ? (
			<>
				<span className="text-red-400 line-through">
					{change.previous_card_name}
				</span>{" "}
				<span className="text-green-400">
					{change.current_card_name}
				</span>
			</>
		) : (
			change.current_card_name
		);

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
			className={clsx(
				`relative grid content-start gap-4 overflow-hidden rounded-lg bg-white/20 pb-5 text-sm`,
				desc_changed ? "col-span-full grid-cols-2" : "",
			)}
		>
			<div
				className={clsx(
					`col-span-full aspect-video w-full overflow-hidden`,
					img_changed ? "rounded-t-lg border-2 border-green-400" : "",
				)}
			>
				{change.current_card_image_url && (
					<img
						className="h-full w-full object-cover"
						loading="lazy"
						src={change.current_card_image_url}
					/>
				)}
			</div>

			<TopBar
				absolute
				changeText="Modified"
				changeColor={clsx("bg-blue-400")}
				tabName={change.card_tab_name}
				link={`${import.meta.env.VITE_ROADMAP_URL}/c/${change.current_card_slug}`}
			/>

			<div className="col-span-full">
				<h3 className="px-5 text-xl">{title}</h3>
			</div>
			{desc_changed && (
				<div className="pl-5">{previous_card_description}</div>
			)}
			<div className={clsx("pr-5", !desc_changed && "pl-5")}>
				{current_card_description}
			</div>
		</div>
	);
}
