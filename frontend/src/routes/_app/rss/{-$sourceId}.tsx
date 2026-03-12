import { Fragment, useState } from "react";

import clsx from "clsx";
import {
	CheckIcon,
	CrossIcon,
	DeleteIcon,
	LinkIcon,
	PlusIcon,
	RefreshIcon,
	TrashIcon,
} from "@storybook/icons";
import { createFileRoute, Link } from "@tanstack/react-router";
import * as v from "valibot";

import { Button, ExternalLink } from "@/components/buttons";
import { formatDate } from "@/components/date";
import {
	useAddSource,
	useEnableSource,
	useDeleteSource,
	useSources,
} from "@/query/sources";
import {
	useClearActivity,
	useInfiniteActivity,
	useRefreshRSS,
} from "@/query/activity";
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
		<div className="mx-auto flex w-90 flex-col gap-2 px-5">
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

function ClearActivity() {
	const { demo } = Route.useSearch();

	const clearActivity = useClearActivity();

	const processing = useProcessing((state) => state.processing);
	const [num, setNum] = useState(0);
	const [error, setError] = useState<string>();

	const loading = clearActivity.isPending || processing;

	return (
		<div className="mx-auto flex w-90 flex-col gap-2 px-5">
			<h3 className="mb-2 text-xl font-bold">
				Clear Activity (0 clears all)
			</h3>
			<form
				onSubmit={async (e) => {
					e.preventDefault();

					if (loading) return;

					setError(undefined);
					updateProcessing(true);
					try {
						await clearActivity.mutateAsync(num);
					} catch (e) {
						console.log("Failed to clear activity", e);
						setError("Failed to clear activity. Check server logs");
					}
					updateProcessing(false);
				}}
			>
				<label
					className="sr-only mb-4 block text-xl"
					htmlFor="clearActivity"
				>
					Number to clear (empty clears all)
				</label>
				<div className="relative flex">
					<input
						value={num}
						disabled={loading}
						onChange={(e) => setNum(Number(e.target.value))}
						className="block w-full rounded-full bg-white/20 py-3 pr-13 pl-4 text-white outline-none focus-visible:border-2 focus-visible:border-white focus-visible:py-2.5 focus-visible:pr-12.5 focus-visible:pl-3.5 disabled:cursor-not-allowed"
						id="clearActivity"
						type="number"
						placeholder="Url"
					/>
					<Button
						Icon={DeleteIcon}
						iconLabel="Add Source"
						disabled={demo || loading}
						animate={clearActivity.isPending}
						error={!!error}
						theme="red"
						invert={false}
						data-isolate
						type="submit"
						className="absolute top-0 right-0"
					/>
				</div>
			</form>
		</div>
	);
}

function RefreshRSS() {
	const { demo } = Route.useSearch();

	const processing = useProcessing((state) => state.processing);

	const refresh = useRefreshRSS();

	return (
		<div className="mx-auto flex w-90 flex-col gap-2 px-5">
			<Button
				Icon={RefreshIcon}
				iconLabel="Refresh RSS"
				disabled={demo || processing}
				animate={refresh.isPending}
				error={refresh.isError}
				onClick={async () => {
					if (processing) return;

					updateProcessing(true);
					try {
						await refresh.mutateAsync();
					} catch (e) {
						console.log("Error refreshing RSS", e);
					}
					updateProcessing(false);
				}}
			>
				Refresh
			</Button>
		</div>
	);
}

function Source({ source }: { source?: TSource }) {
	const { sourceId } = Route.useParams();
	const { demo } = Route.useSearch();

	const enableSource = useEnableSource();
	const deleteSource = useDeleteSource();

	const processing = useProcessing((state) => state.processing);

	return (
		<div
			className={clsx(
				"flex flex-none flex-col items-stretch gap-4 rounded-lg p-5 lg:w-auto",
				sourceId === source?.id && "bg-white/20",
				source ? "w-full max-w-90 lg:max-w-none" : "w-30",
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
			</div>

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
		<div className="flex w-full flex-none gap-2 overflow-x-auto px-4 lg:flex-col lg:px-0">
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

	const {
		data: activity,
		error,
		isLoading,
		hasNextPage,
		isFetchingNextPage,
		fetchNextPage,
	} = useInfiniteActivity(sourceId, demo);

	if (isLoading) {
		return (
			<div className="flex h-full items-center justify-center rounded-lg bg-white/20 p-4">
				<p>Loading</p>
			</div>
		);
	}

	if (
		(error && (!activity || !activity.pages)) ||
		!activity ||
		!activity.pages
	) {
		console.log("Error if error", error);

		return (
			<div className="flex h-full items-center justify-center rounded-lg bg-white/20 p-4">
				<p>Error loading activity</p>
			</div>
		);
	}

	if (activity.pages.length === 0 || activity.pages[0].length === 0) {
		return (
			<div className="flex h-full items-center justify-center rounded-lg bg-white/20 p-4">
				<p>No activity</p>
			</div>
		);
	}

	return (
		<div className="flex h-full flex-col rounded-lg bg-white/20 p-4">
			<div className="flex max-h-full max-w-full flex-col gap-4 overflow-y-auto">
				{activity.pages.map((page, i) => (
					<Fragment key={i}>
						{page.map((post) => (
							<div
								key={post.id}
								className="flex max-w-full items-center justify-between gap-4 not-last:border-b-2 not-last:border-white/20 not-last:pb-4"
							>
								<div className="flex min-w-0 flex-1 flex-wrap gap-1">
									<p className="w-40">
										{formatDate(post.timestamp)}
									</p>
									<p
										className="max-w-full overflow-hidden text-nowrap text-ellipsis"
										title={post.source_url}
									>
										Source: {post.source_url}
									</p>
									<p
										key={post.id}
										className="max-w-full overflow-hidden text-nowrap text-ellipsis"
										title={post.post_url}
									>
										URL: {post.post_url}
									</p>
								</div>
								<ExternalLink
									Icon={LinkIcon}
									iconLabel="View Post"
									href={post.post_url}
									size="small"
								/>
							</div>
						))}
					</Fragment>
				))}

				<Button
					Icon={RefreshIcon}
					iconLabel="Load more activity"
					disabled={!hasNextPage || isFetchingNextPage}
					animate={isFetchingNextPage}
					error={!!error}
					onClick={() =>
						!isFetchingNextPage && hasNextPage && fetchNextPage()
					}
					className="mx-auto min-w-72"
				>
					{isFetchingNextPage
						? "Loading"
						: hasNextPage
							? "Load More"
							: "No More Activity"}
				</Button>
			</div>
		</div>
	);
}

function RouteComponent() {
	return (
		<div className="relative flex flex-col items-center gap-2 pt-24 lg:h-full lg:max-h-full lg:flex-row lg:items-start lg:justify-center lg:gap-0 lg:px-4">
			<div className="flex w-full flex-none flex-col gap-6 overflow-y-auto py-4 lg:max-h-full lg:w-90">
				<AddSource />
				<div className="mx-5 h-0.5 flex-none content-stretch bg-white/20"></div>
				<ClearActivity />
				<div className="mx-5 h-0.5 flex-none content-stretch bg-white/20"></div>
				<RefreshRSS />
				<div className="mx-5 h-0.5 flex-none content-stretch bg-white/20"></div>
				<SourceList />
			</div>
			<div className="w-full overflow-auto px-4 py-4 lg:ml-6 lg:h-full lg:px-0">
				<ActivityList />
			</div>
		</div>
	);
}
