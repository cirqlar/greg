import clsx from "clsx";
import { createFileRoute } from "@tanstack/react-router";

import {
	useRoadmapActivity,
	useRoadmapTabs,
	useRoadmapWatchedTabs,
	useUnwatchTabMutation,
	useWatchTabMutation,
} from "@/query/roadmap";
import type { TRTab, TWatchedTab } from "@/query/types";
import { Button, ExternalLink } from "@/components/buttons";
import { CheckIcon, CrossIcon, LinkIcon, RefreshIcon } from "@storybook/icons";
import { updateProcessing, useProcessing } from "@/stores/processing";
import { useMemo } from "react";
import { useRecheckRSS } from "@/query/activity";

export const Route = createFileRoute("/_app/rmap")({
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
		<div className="flex flex-col gap-2">
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
	} = useRoadmapActivity(demo);

	if (isLoading) {
		return (
			<div className="flex h-full items-center justify-center rounded-lg bg-white/20 p-4">
				<p>Loading</p>
			</div>
		);
	}

	if (error || !roadmapActivity) {
		return (
			<div className="flex h-full items-center justify-center rounded-lg bg-white/20 p-4">
				<p>Error loading activity</p>
			</div>
		);
	}

	if (roadmapActivity.length === 0) {
		return (
			<div className="flex h-full items-center justify-center rounded-lg bg-white/20 p-4">
				<p>No activity</p>
			</div>
		);
	}

	return (
		<div className="h-full rounded-lg bg-white/20 p-4">
			<div className="max-h-full overflow-y-auto">
				{roadmapActivity.map((activity) => (
					<p key={activity.id}>{activity.id}</p>
				))}
			</div>
		</div>
	);
}

function RefreshRoadmap() {
	const { demo } = Route.useSearch();

	const processing = useProcessing((state) => state.processing);

	const refresh = useRecheckRSS();

	return (
		<div className="flex flex-col gap-2 px-5">
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
		<div className="relative flex h-full max-h-full justify-center px-4 pt-24">
			<div className="flex max-h-full w-90 flex-none flex-col gap-6 overflow-y-auto py-4">
				<RefreshRoadmap />
				<div className="mx-5 h-0.5 flex-none content-stretch bg-white/20"></div>
				<TabList />
			</div>
			<div className="ml-6 h-full w-full overflow-auto py-4">
				<ChangeList />
			</div>
		</div>
	);
}
