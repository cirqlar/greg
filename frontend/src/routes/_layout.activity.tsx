import { Fragment, useState } from "react";
import { createFileRoute } from "@tanstack/react-router";
import {
	createColumnHelper,
	flexRender,
	getCoreRowModel,
	useReactTable,
} from "@tanstack/react-table";

import TableGrid from "../components/table-grid";
import { formatDate } from "../components/date";
import type { TActivity } from "../query/types";
import {
	useActivity,
	useClearActivities,
	useRecheckRSS,
} from "../query/activity";

export const Route = createFileRoute("/_layout/activity")({
	component: Activity,
});

const columnHelper = createColumnHelper<TActivity>();
const columns = [
	columnHelper.accessor("id", {
		cell: (info) => info.getValue(),
		header: "ID",
	}),
	columnHelper.accessor("post_url", {
		cell: (info) => (
			<span className="wrap-break-word">
				<a
					href={info.getValue()}
					target="_blank"
					referrerPolicy="no-referrer"
				>
					{info.getValue()}
				</a>
			</span>
		),
		header: "Post",
	}),
	columnHelper.accessor("source_url", {
		cell: (info) => (
			<span className="wrap-break-word">
				<a
					href={info.getValue()}
					target="_blank"
					referrerPolicy="no-referrer"
				>
					{info.getValue()}
				</a>
			</span>
		),
		header: "Source",
	}),
	columnHelper.accessor("timestamp", {
		cell: (info) => (
			<span className="wrap-break-word">
				{formatDate(info.getValue())}
			</span>
		),
		header: "Checked At",
	}),
];

function Activity() {
	const [loading, setLoading] = useState(false);
	const [num, setNum] = useState(0);

	const activity = useActivity();

	const table = useReactTable({
		columns: columns,
		data: activity.data ?? [],
		getCoreRowModel: getCoreRowModel(),
	});

	const recheck = useRecheckRSS();

	const clearActivities = useClearActivities();

	return (
		<>
			<div className="mx-auto mb-8 flex max-w-lg justify-end px-4">
				<button
					disabled={loading}
					className="h-full rounded-sm bg-green-700 px-4 py-3 font-bold uppercase"
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
			<div className="mx-auto mb-8 max-w-lg px-4">
				<h3 className="mb-4 text-2xl font-bold">Clear Activity</h3>
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
					<label className="mb-4 block text-xl" htmlFor="password">
						Number to clear (empty clears all)
					</label>
					<div className="flex">
						<input
							value={num}
							disabled={loading}
							onChange={(e) => setNum(Number(e.target.value))}
							className="mr-2 block w-full rounded-sm px-4 py-3 text-black"
							id="password"
							type="number"
							placeholder="Url"
						/>
						<button
							disabled={loading}
							className="h-full rounded-sm bg-green-700 px-4 py-3 font-bold uppercase"
							type="submit"
						>
							Clear
						</button>
					</div>
				</form>
			</div>
			<div className="mx-auto max-w-4xl px-4">
				<h3 className="mb-4 text-2xl font-bold">Activity</h3>
				{activity.isError ? (
					<p>There's been an error fetching activity</p>
				) : activity.isPending ? (
					<p>Fetching activity...</p>
				) : activity.data.length == 0 ? (
					<p>There is no saved activity</p>
				) : (
					<TableGrid>
						<Fragment>
							{table.getHeaderGroups().map((headerGroup) => (
								<Fragment key={headerGroup.id}>
									{headerGroup.headers.map((header) => (
										<div key={header.id}>
											{header.isPlaceholder
												? null
												: flexRender(
														header.column.columnDef
															.header,
														header.getContext(),
													)}
										</div>
									))}
								</Fragment>
							))}
						</Fragment>
						<Fragment>
							{table.getRowModel().rows.map((row) => (
								<Fragment key={row.id}>
									{row.getVisibleCells().map((cell) => (
										<div key={cell.id}>
											{flexRender(
												cell.column.columnDef.cell,
												cell.getContext(),
											)}
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
