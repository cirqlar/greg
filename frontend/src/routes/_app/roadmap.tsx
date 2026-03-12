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
	useRoadmapWatchedTabs,
	useUnwatchTabMutation,
	useWatchTabMutation,
} from "@/query/roadmap";
import type { TRTab, TWatchedTab } from "@/query/types";
import { updateProcessing, useProcessing } from "@/stores/processing";

export const Route = createFileRoute("/_app/roadmap")({
	component: RouteComponent,
});

type ProcessedTab = TRTab &
	(
		| { watched: true; watchedTab: TWatchedTab }
		| { watched: false; watchedTab?: never }
	);

function Tab({ tab }: { tab: ProcessedTab }) {
	const { demo } = Route.useSearch();

	const watchTab = useWatchTabMutation();
	const unwatchTab = useUnwatchTabMutation();

	const processing = useProcessing((state) => state.processing);

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
				<div
					className={clsx(
						"flex items-center rounded-full px-3.5 py-1 text-sm text-black",
						tab.watched ? "bg-green-400" : "bg-red-400",
					)}
				>
					<p>{tab.watched ? "Watched" : "Not Watched"}</p>
				</div>
				<div className="flex gap-2">
					<ExternalLink
						Icon={LinkIcon}
						iconLabel="Open Source"
						href={`${import.meta.env.VITE_ROADMAP_URL}/tabs/${tab.slug}`}
						size="small"
					/>
					<Button
						Icon={tab.watched ? CrossIcon : CheckIcon}
						iconLabel={tab.watched ? "Unwatch Tab" : "Watch Tab"}
						disabled={demo || processing}
						animate={watchTab.isPending || unwatchTab.isPending}
						error={watchTab.isError || unwatchTab.isError}
						theme={tab.watched ? "red" : "green"}
						size="small"
						onClick={async () => {
							if (processing) return;

							updateProcessing(true);
							try {
								if (tab.watched) {
									await unwatchTab.mutateAsync(
										tab.watchedTab.id,
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
						{tab.watched ? "Unwatch" : "Watch"}
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
	const {
		data: watchedTabs,
		error: wtError,
		isLoading: wtIsLoading,
	} = useRoadmapWatchedTabs(demo);

	const processedTabs = useMemo(() => {
		if (roadmapTabs && watchedTabs) {
			const tabs: ProcessedTab[] = [];

			for (let i = 0; i < watchedTabs.length; i++) {
				const wTab = watchedTabs[i];

				let tab = roadmapTabs.find((t) => t.id == wTab.tab_id);

				if (!tab) {
					console.log("Missing Watched tab", wTab.tab_id);
					tab = {
						db_id: 0,
						name: "Missing Tab",
						slug: "Missing Slug",
						id: wTab.tab_id,
					};
				}

				tabs.push({ ...tab, watched: true, watchedTab: wTab });
			}

			for (let i = 0; i < roadmapTabs.length; i++) {
				let tab = roadmapTabs[i];
				if (!watchedTabs.find((wTab) => wTab.tab_id === tab.id)) {
					tabs.push({ ...tab, watched: false });
				}
			}

			return tabs;
		} else {
			return [];
		}
	}, [roadmapTabs, watchedTabs]);

	if (rtIsLoading || wtIsLoading) {
		return (
			<div className="px-5">
				<p>Loading</p>
			</div>
		);
	}

	if (rtError || !roadmapTabs || wtError || !watchedTabs) {
		return (
			<div className="px-5">
				<p>Error loading roadmap tabs</p>
			</div>
		);
	}

	if (processedTabs.length === 0) {
		return (
			<div className="px-5">
				<p>No tabs yet</p>
			</div>
		);
	}

	return (
		<div className="grid w-full grid-cols-[repeat(auto-fit,minmax(320px,1fr))] gap-2 lg:flex lg:flex-col">
			{processedTabs.map((tab) => (
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
									(activity.change_count &&
										activity.change_count !== 0) ||
									!hideEmpty,
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
