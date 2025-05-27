import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createFileRoute, Link } from "@tanstack/react-router";
import {
	createColumnHelper,
	flexRender,
	getCoreRowModel,
	useReactTable,
} from "@tanstack/react-table";
import { Fragment, useMemo, useState } from "react";
import TableGrid from "../components/table-grid";

import style from "../components/table-grid.module.css";
import { formatDate } from "../components/date";

export const Route = createFileRoute("/_layout/roadmaps")({
	component: Roadmaps,
});

type TRoadmapActivity = {
	id: number;
	timestamp: string;
};
type TWatchedTab = {
	id: number;
	tab_id: string;
	timestamp: string;
};
type TRTab = {
	id: string;
	name: string;
	slug: string;
	db_id: number;
};

const roadmapColumnHelper = createColumnHelper<TRoadmapActivity>();
const watchedTabColumnHelper = createColumnHelper<TWatchedTab>();
const tabColumnHelper = createColumnHelper<TRTab>();

const roadmapColumns = [
	roadmapColumnHelper.accessor("id", {
		cell: (info) => info.getValue(),
		header: "ID",
	}),
	roadmapColumnHelper.accessor("timestamp", {
		cell: (info) => (
			<span className="break-words">{formatDate(info.getValue())}</span>
		),
		header: "Saved At",
	}),
	roadmapColumnHelper.display({
		id: "viewChanges",
		cell: (props) => (
			<Link
				to="/roadmap/$roadmap_id"
				params={{ roadmap_id: props.row.original.id.toString() }}
				className="bg-green-700 text-inherit inline-block px-4 py-3 uppercase font-bold rounded"
			>
				Changes
			</Link>
		),
	}),
];

function useWatchedTabsTable(roadmapWatchedTabs: TWatchedTab[]| undefined, mostRecentTabs: TRTab[]| undefined) {
	const queryClient = useQueryClient();
	const [loading, setLoading] = useState(false);

	const deleteWatchedTab = useMutation({
		mutationFn: (id: number) =>
			fetch(`/api/watched_tabs/${id}`, {
				method: "DELETE",
			}).then((res) => res.json()),
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ["watched_tabs"] });
		},
	});

	const watchedTabColumns = useMemo(
		() => [
			watchedTabColumnHelper.accessor("id", {
				cell: (info) => info.getValue(),
				header: "ID",
			}),
			watchedTabColumnHelper.accessor("tab_id", {
				cell: (info) => {
					const tab_id = info.getValue();
					const tab_name =
						(mostRecentTabs ?? []).find((t) => t.id === tab_id)?.name ??
						"Missing Tab";

					return <span className="break-words">{tab_name}</span>;
				},
				header: "Tab Name",
			}),
			watchedTabColumnHelper.accessor("timestamp", {
				cell: (info) => (
					<span className="break-words">{formatDate(info.getValue())}</span>
				),
				header: "Added At",
			}),

			watchedTabColumnHelper.display({
				id: "removeTab",
				cell: (props) => (
					<button
						disabled={loading}
						className="bg-green-700 px-4 py-3 uppercase font-bold rounded"
						onClick={async (e) => {
							e.preventDefault();

							setLoading(true);
							try {
								await deleteWatchedTab.mutateAsync(props.row.original.id)
							} catch  {
								console.log("Delete watched tab failed");
							}
							setLoading(false);
						}}
					>
						Remove
					</button>
				),
			}),
		],
		[deleteWatchedTab, loading, mostRecentTabs]
	);

	const watchedTabsTable = useReactTable({
		columns: watchedTabColumns,
		data: roadmapWatchedTabs ?? [],
		getCoreRowModel: getCoreRowModel(),
	});

	return watchedTabsTable;
}

function useMostRecentTabsTable(mostRecentTabs: TRTab[]| undefined, roadmapWatchedTabs: TWatchedTab[]| undefined) {
	const queryClient = useQueryClient();
	const [loading, setLoading] = useState(false);

	const addWatchedTab = useMutation({
		mutationFn: (tab_id: string) =>
			fetch(`/api/watched_tabs/add/${tab_id}`, {
				method: "POST",
			}).then((res) => res.json()),
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ["watched_tabs"] });
		},
	});

	const tabColumns = useMemo(
		() => [
			tabColumnHelper.accessor("id", {
				cell: (info) => <span className="break-words">{info.getValue()}</span>,
				header: "ID",
			}),
			tabColumnHelper.accessor("name", {
				cell: (info) => <span className="break-words">{info.getValue()}</span>,
				header: "Name",
			}),
			tabColumnHelper.accessor("slug", {
				cell: (info) => <span className="break-words">{info.getValue()}</span>,
				header: "Slug",
			}),
			tabColumnHelper.display({
				id: "addToWatchedTabs",
				cell: (props) => {
					const isWatched = (roadmapWatchedTabs ?? []).some(
						(t) => t.tab_id === props.row.original.id
					);

					return (
						<button
							disabled={isWatched || loading}
							className="bg-green-700 px-4 py-3 uppercase font-bold rounded"
							onClick={async (e) => {
								e.preventDefault();

								setLoading(true);
								try {

									await addWatchedTab.mutateAsync(props.row.original.id);
								} catch {
									console.log("Add Watched Tab Failed");
								}
								setLoading(false)
							}}
						>
							Watch
						</button>
					);
				},
			}),
		],
		[addWatchedTab, loading, roadmapWatchedTabs]
	);

	const tabsTable = useReactTable({
		columns: tabColumns,
		data: mostRecentTabs ?? [],
		getCoreRowModel: getCoreRowModel(),
	});

	return tabsTable;
}

function Roadmaps() {
	const [loading, setLoading] = useState(false);

	const queryClient = useQueryClient();
	const roadmapActivity = useQuery<TRoadmapActivity[]>({
		queryKey: ["roadmap_activity"],
		queryFn: () => fetch("/api/roadmap_activity").then(async (res) => {
			if (res.ok) {
				return res.json();
			} else {
				let err;
				try {
					err = await res.json();
				} catch {
					err = "Non-json return from response";
				}
				console.log("Error fetching roadmap activity", err);
				throw err;
			}
		}),
	});
	const mostRecentTabs = useQuery<TRTab[]>({
		queryKey: ["most_recent_tabs"],
		queryFn: () => fetch("/api/most_recent_tabs").then(async (res) => {
			if (res.ok) {
				return res.json();
			} else {
				let err;
				try {
					err = await res.json();
				} catch {
					err = "Non-json return from response";
				}
				console.log("Error fetching most recent tabs", err);
				throw err;
			}
		}),
	});
	const roadmapWatchedTabs = useQuery<TWatchedTab[]>({
		queryKey: ["watched_tabs"],
		queryFn: () => fetch("/api/watched_tabs").then(async (res) => {
			if (res.ok) {
				return res.json();
			} else {
				let err;
				try {
					err = await res.json();
				} catch {
					err = "Non-json return from response";
				}
				console.log("Error fetching watched tabs", err);
				throw err;
			}
		}),
	});

	const roadmapTable = useReactTable({
		columns: roadmapColumns,
		data: roadmapActivity.data ?? [],
		getCoreRowModel: getCoreRowModel(),
	});

	const watchedTabsTable = useWatchedTabsTable(roadmapWatchedTabs.data, mostRecentTabs.data);
	const tabsTable = useMostRecentTabsTable(mostRecentTabs.data, roadmapWatchedTabs.data);

	const recheck = useMutation({
		mutationFn: () =>
			fetch("/api/recheck_roadmap", {
				method: "POST",
			}).then((res) => res.json()),
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ["roadmap_activity"] });
			queryClient.invalidateQueries({ queryKey: ["most_recent_tabs"] });
			queryClient.invalidateQueries({ queryKey: ["watched_tabs"] });
		},
	});

	// const clearActivities = useMutation({
	//   mutationFn: (num: number) =>
	//     fetch(`/api/activity${num < 1 ? "" : `/${num}`}`, {
	//       method: "DELETE",
	//     }).then((res) => res.json()),
	//   onSuccess: () => {
	//     queryClient.invalidateQueries({ queryKey: ["activity"] });
	//   },
	// });

	return (
		<>
			<div className="max-w-lg mx-auto px-4 mb-8 flex justify-end">
				<button
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
				</button>
			</div>

			<div className="max-w-4xl mb-8 mx-auto px-4">
				<h3 className="text-2xl font-bold mb-4">Watched Tabs</h3>
				{roadmapWatchedTabs.isError ?
					<p>There's been an error fetching watched tabs</p>
				: roadmapWatchedTabs.isPending ?
					<p>Fetching watched tabs...</p>
				: roadmapWatchedTabs.data.length === 0 ?
					<p>There are no saved watched tabs</p>
				: (
					<TableGrid>
						<Fragment>
							{watchedTabsTable.getHeaderGroups().map((headerGroup) => (
								<Fragment key={headerGroup.id}>
									{headerGroup.headers.map((header) => (
										<div key={header.id}>
											{header.isPlaceholder
												? null
												: flexRender(
														header.column.columnDef.header,
														header.getContext()
													)}
										</div>
									))}
								</Fragment>
							))}
						</Fragment>
						<Fragment>
							{watchedTabsTable.getRowModel().rows.map((row) => (
								<Fragment key={row.id}>
									{row.getVisibleCells().map((cell) => (
										<div key={cell.id}>
											{flexRender(cell.column.columnDef.cell, cell.getContext())}
										</div>
									))}
								</Fragment>
							))}
						</Fragment>
					</TableGrid>
				)}
			</div>

			<div className="max-w-4xl mb-8 mx-auto px-4">
				<h3 className="text-2xl font-bold mb-4">Most Recent Tabs</h3>
				{mostRecentTabs.isError ?
					<p>There's been an error fetching most recent tabs</p>
				: mostRecentTabs.isPending ?
					<p>Fetching most recent tabs...</p>
				: mostRecentTabs.data.length == 0 ?
					<p>There are no saved most recent tabs</p>
				: (
					<TableGrid>
						<Fragment>
							{tabsTable.getHeaderGroups().map((headerGroup) => (
								<Fragment key={headerGroup.id}>
									{headerGroup.headers.map((header) => (
										<div key={header.id}>
											{header.isPlaceholder
												? null
												: flexRender(
														header.column.columnDef.header,
														header.getContext()
													)}
										</div>
									))}
								</Fragment>
							))}
						</Fragment>
						<Fragment>
							{tabsTable.getRowModel().rows.map((row) => (
								<Fragment key={row.id}>
									{row.getVisibleCells().map((cell) => (
										<div key={cell.id}>
											{flexRender(cell.column.columnDef.cell, cell.getContext())}
										</div>
									))}
								</Fragment>
							))}
						</Fragment>
					</TableGrid>
				)}
			</div>

			<div className="max-w-4xl mb-8 mx-auto px-4">
				<h3 className="text-2xl font-bold mb-4">Roadmap Activity</h3>
				{roadmapActivity.isError ?
					<p>There's been an error fetching roadmap activity</p>
				: roadmapActivity.isPending ?
					<p>Fetching roadmap activity...</p>
				: roadmapActivity.data.length === 0 ?
					<p>There is no saved roadmap activity</p>
				: (
					<TableGrid className={style.template_2}>
						<Fragment>
							{roadmapTable.getHeaderGroups().map((headerGroup) => (
								<Fragment key={headerGroup.id}>
									{headerGroup.headers.map((header) => (
										<div key={header.id}>
											{header.isPlaceholder
												? null
												: flexRender(
														header.column.columnDef.header,
														header.getContext()
													)}
										</div>
									))}
								</Fragment>
							))}
						</Fragment>
						<Fragment>
							{roadmapTable.getRowModel().rows.map((row) => (
								<Fragment key={row.id}>
									{row.getVisibleCells().map((cell) => (
										<div key={cell.id}>
											{flexRender(cell.column.columnDef.cell, cell.getContext())}
										</div>
									))}
								</Fragment>
							))}
						</Fragment>
					</TableGrid>
				)}
			</div>
		</>
	);
}
