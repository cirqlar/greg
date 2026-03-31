import { Fragment, useMemo, useState } from "react";

import clsx from "clsx";
import {
	CheckIcon,
	CrossIcon,
	EyeIcon,
	LinkIcon,
	RefreshIcon,
} from "@storybook/icons";
import { createFileRoute } from "@tanstack/react-router";

import { Button, ExternalLink, Link } from "@/components/buttons";
import { formatDate } from "@/components/date";
import {
	useInfiniteRoadmapActivity,
	useRefreshRoadmap,
	useRoadmapTabs,
	useUnwatchTabMutation,
	useWatchTabMutation,
} from "@/query/roadmap";
import type { TRTab } from "@/query/types";
import { updateProcessing, useProcessing } from "@/stores/processing";

export const Route = createFileRoute("/_app/roadmap")({
	component: RouteComponent,
});

function Tab({ tab }: { tab: TRTab }) {
	const { demo } = Route.useSearch();

	const watchTab = useWatchTabMutation();
	const unwatchTab = useUnwatchTabMutation();

	const processing = useProcessing((state) => state.processing);

	const watched = tab.watch_id !== null;

	return (
		<div className="flex flex-col items-stretch gap-4 rounded-lg px-5 py-3">
			<div className="flex items-center justify-between gap-2">
				<p
					className="flex-1 overflow-hidden text-lg text-nowrap text-ellipsis"
					title={tab.name}
				>
					{tab.name}
				</p>
			</div>

			<div className="flex items-stretch justify-between">
				<div className="flex">
					{watched && (
						<div
							className={clsx(
								"flex items-center rounded-full bg-green-400 px-3.5 py-1 text-sm text-black",
							)}
						>
							<p>Watched</p>
						</div>
					)}
					{tab.deleted && (
						<div
							className={clsx(
								"flex items-center rounded-full bg-red-400 px-3.5 py-1 text-sm text-black",
							)}
						>
							<p>Deleted</p>
						</div>
					)}
					{!watched && !tab.deleted && (
						<div
							className={clsx(
								"flex items-center rounded-full bg-white px-3.5 py-1 text-sm text-black",
							)}
						>
							<p>Not Watched</p>
						</div>
					)}
				</div>
				<div className="flex gap-2">
					<ExternalLink
						Icon={LinkIcon}
						iconLabel="Open Source"
						href={`${import.meta.env.VITE_ROADMAP_URL}/tabs/${tab.slug}`}
						size="small"
					/>
					<Button
						Icon={watched ? CrossIcon : CheckIcon}
						iconLabel={watched ? "Unwatch Tab" : "Watch Tab"}
						disabled={demo || processing}
						animate={watchTab.isPending || unwatchTab.isPending}
						error={watchTab.isError || unwatchTab.isError}
						theme={watched ? "red" : "green"}
						size="small"
						onClick={async () => {
							if (processing) return;

							updateProcessing(true);
							try {
								if (watched) {
									// can't be watched if its null
									await unwatchTab.mutateAsync(
										tab.watch_id as number,
									);
								} else {
									await watchTab.mutateAsync(tab.id);
								}
							} catch (e) {
								console.log("Error deleting source", e);
							}
							updateProcessing(false);
						}}
					>
						{watched ? "Unwatch" : "Watch"}
					</Button>
				</div>
			</div>
		</div>
	);
}

function TabList() {
	const { demo } = Route.useSearch();

	const {
		data: roadmapTabs,
		error: rtError,
		isLoading: rtIsLoading,
	} = useRoadmapTabs(demo);

	const sortedTabs = useMemo(() => {
		if (roadmapTabs) {
			return [...roadmapTabs].sort((a, b) => {
				const a_watched = a.watch_id !== null;
				const b_watched = b.watch_id !== null;

				if (a_watched === b_watched && a.deleted === b.deleted) {
					return a.name.localeCompare(b.name);
				} else if (a_watched && !b_watched) {
					return -1;
				} else if (!a_watched && b_watched) {
					return 1;
				} else if (!a.deleted && b.deleted) {
					return -1;
				} else if (a.deleted && !b.deleted) {
					return 1;
				}

				throw new Error("unreachable");
			});
		} else {
			return [];
		}
	}, [roadmapTabs]);

	if (rtIsLoading) {
		return (
			<div className="px-5">
				<p>Loading</p>
			</div>
		);
	}

	if (rtError || !roadmapTabs) {
		return (
			<div className="px-5">
				<p>Error loading roadmap tabs</p>
			</div>
		);
	}

	if (sortedTabs.length === 0) {
		return (
			<div className="px-5">
				<p>No tabs yet</p>
			</div>
		);
	}

	return (
		<div className="grid w-full grid-cols-[repeat(auto-fit,minmax(320px,1fr))] gap-2 lg:flex lg:flex-col">
			{sortedTabs.map((tab) => (
				<Tab tab={tab} key={tab.name} />
			))}
		</div>
	);
}

function ChangeList() {
	const { demo } = Route.useSearch();

	const {
		data: roadmapActivity,
		error,
		isLoading,
		hasNextPage,
		isFetchingNextPage,
		fetchNextPage,
	} = useInfiniteRoadmapActivity(demo);

	const [hideEmpty, setHideEmpty] = useState(true);

	if (isLoading) {
		return (
			<div className="flex h-full items-center justify-center rounded-lg bg-white/20 p-4">
				<p>Loading</p>
			</div>
		);
	}

	if (
		(error && (!roadmapActivity || !roadmapActivity.pages)) ||
		!roadmapActivity ||
		!roadmapActivity.pages
	) {
		return (
			<div className="flex h-full items-center justify-center rounded-lg bg-white/20 p-4">
				<p>Error loading activity</p>
			</div>
		);
	}

	if (
		roadmapActivity.pages.length === 0 ||
		roadmapActivity.pages[0].length === 0
	) {
		return (
			<div className="flex h-full items-center justify-center rounded-lg bg-white/20 p-4">
				<p>No activity</p>
			</div>
		);
	}

	return (
		<div className="flex h-full flex-col gap-4 rounded-lg bg-white/20 p-4">
			<div className="flex items-center justify-end gap-2">
				<input
					checked={hideEmpty}
					onChange={(e) => setHideEmpty(e.target.checked)}
					type="checkbox"
					name="filterEmpty"
					id="filterEmpty"
				/>
				<label htmlFor="filterEmpty">Hide Empty Changes</label>
			</div>
			<div className="flex max-h-full flex-col gap-2 overflow-y-auto">
				{roadmapActivity.pages.map((page, i) => (
					<Fragment key={i}>
						{page
							.filter(
								(activity) =>
									activity.change_count || !hideEmpty,
							)
							.map((activity) => (
								<div
									key={activity.id}
									className="flex items-center justify-between not-last:border-b-2 not-last:border-white/20 not-last:pb-2"
								>
									<p className="w-40">
										{formatDate(activity.timestamp)}
									</p>
									<p className="w-24">
										{activity.change_count ?? 0}{" "}
										{activity.change_count === 1
											? "Change"
											: "Changes"}
									</p>

									<Link
										to="/roadmap/$roadmapId"
										params={{ roadmapId: activity.id }}
										search={(prev) => prev}
										iconLabel="View Changes"
										Icon={EyeIcon}
										size="small"
									>
										View
									</Link>
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

function RefreshRoadmap() {
	const { demo } = Route.useSearch();

	const processing = useProcessing((state) => state.processing);

	const refresh = useRefreshRoadmap();

	return (
		<div className="mx-auto flex w-90 flex-col gap-2 px-5">
			<Button
				Icon={RefreshIcon}
				iconLabel="Refresh Roadmap"
				disabled={demo || processing}
				animate={refresh.isPending}
				error={refresh.isError}
				onClick={async () => {
					if (processing) return;

					updateProcessing(true);
					try {
						await refresh.mutateAsync();
					} catch (e) {
						console.log("Error refreshing roadmap", e);
					}
					updateProcessing(false);
				}}
			>
				Refresh
			</Button>
		</div>
	);
}

function RouteComponent() {
	return (
		<div className="relative flex flex-col items-center gap-6 pt-24 lg:h-full lg:max-h-full lg:flex-row lg:items-start lg:justify-center lg:gap-0 lg:px-4">
			<div className="flex w-full flex-none flex-col gap-6 overflow-y-auto py-4 lg:max-h-full lg:w-90">
				<RefreshRoadmap />
				<div className="mx-5 h-0.5 flex-none content-stretch bg-white/20"></div>
				<TabList />
			</div>
			<div className="h-full w-full overflow-auto px-4 py-4 lg:ml-6 lg:px-0">
				<ChangeList />
			</div>
		</div>
	);
}
