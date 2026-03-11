import { useState } from "react";

import clsx from "clsx";
import {
	CheckIcon,
	CrossIcon,
	LinkIcon,
	PlusIcon,
	TrashIcon,
} from "@storybook/icons";
import { createFileRoute, Link } from "@tanstack/react-router";
import * as v from "valibot";

import { Button, ExternalLink } from "@/components/buttons";
import {
	useAddSource,
	useEnableSource,
	useDeleteSource,
	useSources,
} from "@/query/sources";
import { useActivity } from "@/query/activity";
import type { TSource } from "@/query/types";
import { updateProcessing, useProcessing } from "@/stores/processing";

export const Route = createFileRoute("/_app/rss/{-$sourceId}")({
	component: RouteComponent,
	params: {
		parse: (rawParams) =>
			v.parse(
				v.object({
					sourceId: v.optional(
						v.pipe(
							v.string(),
							v.trim(),
							v.nonEmpty(),
							v.toNumber(),
						),
						undefined,
					),
				}),
				rawParams,
			),
	},
});

const urlSchema = v.pipe(
	v.string(),
	v.trim(),
	v.nonEmpty("Please enter a URL"),
	v.url("Please enter a valid URL"),
);

function AddSource() {
	const { demo } = Route.useSearch();

	const addSource = useAddSource();

	const processing = useProcessing((state) => state.processing);
	const [url, setUrl] = useState("");
	const [urlError, setUrlError] = useState<string>();

	const loading = addSource.isPending || processing;

	return (
		<div className="flex flex-col gap-2 px-5">
			<h3 className="mb-2 text-xl font-bold">Add Source</h3>
			<form
				onSubmit={async (e) => {
					e.preventDefault();
					if (loading) return;

					setUrlError(undefined);
					updateProcessing(true);
					try {
						const validURL = v.parse(urlSchema, url);
						await addSource.mutateAsync({ url: validURL });
					} catch (e) {
						if (v.isValiError(e)) {
							setUrlError(e.message);
						} else {
							console.log("adding source failed", e);
							setUrlError(
								"Adding source failed. Check server logs",
							);
						}
					}
					updateProcessing(false);
				}}
			>
				<label
					className="sr-only mb-4 block text-xl"
					htmlFor="addSource"
				>
					URL
				</label>
				<div className="relative flex">
					<input
						disabled={demo || loading}
						onChange={(e) => setUrl(e.target.value)}
						className="block w-full rounded-full bg-white/20 py-3 pr-13 pl-4 text-white outline-none focus-visible:border-2 focus-visible:border-white focus-visible:py-2.5 focus-visible:pr-12.5 focus-visible:pl-3.5 disabled:cursor-not-allowed"
						id="addSource"
						type="url"
						placeholder="Url"
					/>

					<Button
						Icon={PlusIcon}
						iconLabel="Add Source"
						disabled={demo || loading}
						animate={addSource.isPending}
						error={!!urlError}
						data-isolate
						type="submit"
						className="absolute top-0 right-0"
					/>
				</div>
			</form>
			{urlError && (
				<p className="text-center text-sm text-red-500">{urlError}</p>
			)}
		</div>
	);
}

function Source({ source }: { source?: TSource }) {
	const { sourceId } = Route.useParams();
	const { demo } = Route.useSearch();

	const enableSource = useEnableSource();
	const deleteSource = useDeleteSource();

	const processing = useProcessing((state) => state.processing);

	// const domain = source ? new URL(source.url).hostname : undefined;

	return (
		<div
			className={clsx(
				"flex flex-col items-stretch gap-4 rounded-lg px-5 py-3",
				sourceId === source?.id && "bg-white/20",
			)}
		>
			<div className="flex items-center justify-between gap-2">
				<p
					className={clsx(
						"flex-1 overflow-hidden text-lg text-nowrap text-ellipsis",
						sourceId === source?.id ? "font-bold" : "",
					)}
					title={source?.url}
				>
					<Link
						to="/rss/{-$sourceId}"
						params={{ sourceId: source?.id }}
						search={(prev) => prev}
						className="text-white"
					>
						{source ? source.url : "All"}{" "}
					</Link>
				</p>
				{/* {source && (
					<p
						className={clsx(
							"rounded-full px-2 py-1 text-sm text-black",
							source.enabled ? "bg-green-400" : "bg-red-400",
						)}
					>
						{source.enabled ? "Enabled" : "Disabled"}
					</p>
				)} */}
			</div>
			{/* <div className="flex min-w-0 flex-1 flex-col gap-2">
				{source && (
					<a
						href={source.url}
						target="_blank"
						referrerPolicy="no-referrer"
						className="flex items-center gap-2 text-sm"
					>
						<LinkIcon className="flex-none" />
						<span className="min-w-0 flex-1 overflow-hidden text-nowrap text-ellipsis">
							{source.url}
						</span>
					</a>
				)}
			</div> */}

			{source && (
				<div className="flex items-stretch justify-between">
					<div
						className={clsx(
							"flex items-center rounded-full px-3.5 py-1 text-sm text-black",
							source.enabled ? "bg-green-400" : "bg-red-400",
						)}
					>
						<p>{source.enabled ? "Enabled" : "Disabled"}</p>
					</div>
					<div className="flex gap-2">
						<ExternalLink
							Icon={LinkIcon}
							iconLabel="Open Source"
							href={source.url}
							size="small"
						/>
						<Button
							Icon={source.enabled ? CrossIcon : CheckIcon}
							iconLabel={
								source.enabled
									? "Disable Source"
									: "Enable source"
							}
							disabled={demo || processing}
							animate={enableSource.isPending}
							error={enableSource.isError}
							theme={source.enabled ? "red" : "green"}
							size="small"
							onClick={async () => {
								if (processing) return;

								updateProcessing(true);
								try {
									await enableSource.mutateAsync({
										id: source.id,
										enable: !source.enabled,
									});
								} catch (e) {
									console.log("Error deleting source", e);
								}
								updateProcessing(false);
							}}
						>
							{source.enabled ? "Disable" : "Enable"}
						</Button>
						<Button
							Icon={TrashIcon}
							iconLabel="Delete source"
							disabled={demo || processing}
							animate={deleteSource.isPending}
							error={deleteSource.isError}
							theme="red"
							size="small"
							onClick={async () => {
								if (processing) return;

								updateProcessing(true);
								try {
									await deleteSource.mutateAsync(source.id);
								} catch (e) {
									console.log("Error deleting source", e);
								}
								updateProcessing(false);
							}}
						/>
					</div>
				</div>
			)}
		</div>
	);
}

function SourceList() {
	const { demo } = Route.useSearch();

	const { data: sources, error, isLoading } = useSources(demo);

	if (isLoading) {
		return (
			<div className="px-5">
				<p>Loading</p>
			</div>
		);
	}

	if (error || !sources) {
		return (
			<div className="px-5">
				<p>Error loading sources</p>
			</div>
		);
	}

	if (sources.length === 0) {
		return (
			<div className="px-5">
				<p>No sources. Add one</p>
			</div>
		);
	}

	return (
		<div className="flex flex-col gap-2">
			<Source />
			{sources.map((source) => (
				<Source source={source} key={source.id} />
			))}
		</div>
	);
}

function ActivityList() {
	const { sourceId } = Route.useParams();
	const { demo } = Route.useSearch();

	const { data: activity, error, isLoading } = useActivity(sourceId, demo);

	if (isLoading) {
		return (
			<div className="flex h-full items-center justify-center rounded-lg bg-white/20 p-4">
				<p>Loading</p>
			</div>
		);
	}

	if (error || !activity) {
		return (
			<div className="flex h-full items-center justify-center rounded-lg bg-white/20 p-4">
				<p>Error loading activity</p>
			</div>
		);
	}

	if (activity.length === 0) {
		return (
			<div className="flex h-full items-center justify-center rounded-lg bg-white/20 p-4">
				<p>No activity</p>
			</div>
		);
	}

	return (
		<div className="h-full rounded-lg bg-white/20 p-4">
			<div className="max-h-full overflow-y-auto">
				{activity.map((post) => (
					<p key={post.id}>{post.post_url}</p>
				))}
			</div>
		</div>
	);
}

function RouteComponent() {
	return (
		<div className="relative flex h-full max-h-full justify-center px-4 pt-24">
			<div className="flex max-h-full w-90 flex-none flex-col gap-6 overflow-y-auto py-4">
				<AddSource />
				<div className="mx-5 h-0.5 flex-none content-stretch bg-white/20"></div>
				<SourceList />
			</div>
			<div className="ml-6 h-full w-full overflow-auto py-4">
				<ActivityList />
			</div>
		</div>
	);
}
