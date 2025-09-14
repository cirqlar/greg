import { Fragment, useMemo, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import {
	createColumnHelper,
	flexRender,
	getCoreRowModel,
	useReactTable,
} from "@tanstack/react-table";
import TableGrid from "../components/table-grid";

import style from "../components/table-grid.module.css";
import { formatDate } from "../components/date";

export const Route = createFileRoute("/_layout/sources")({
	component: Sources,
});

type TSource = { id: number; url: string; last_checked: string, enabled: boolean };
const columnHelper = createColumnHelper<TSource>();

function Sources() {
	const [loading, setLoading] = useState(false);

	const [url, setUrl] = useState("");
	const queryClient = useQueryClient();
	const sources = useQuery<TSource[]>({
		queryKey: ["sources"],
		queryFn: () => fetch("/api/sources").then(async (res) => {
			if (res.ok) {
				return res.json();
			} else {
				let err;
				try {
					err = await res.json();
				} catch {
					err = "Non-json return from response";
				}
				console.log("Error fetching sources", err);
				throw err;
			}
		}),
	});

	const addSource = useMutation({
		mutationFn: (source: { url: string }) =>
			fetch("/api/source/new", {
				method: "POST",
				headers: {
					"Content-Type": "application/json",
				},
				body: JSON.stringify(source),
			}).then((res) => res.json()),
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ["sources"] });
		},
	});
	
	const enableSource = useMutation({
		mutationFn: ({id, enable}: {id: number, enable: boolean}) =>
			fetch(`/api/source/${id}/enable/${enable}`, {
				method: "POST",
				headers: {
					"Content-Type": "application/json",
				},
				// body: JSON.stringify(source),
			}).then((res) => res.json()),
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ["sources"] });
		},
	});

	const deleteSource = useMutation({
		mutationFn: (id: number) =>
			fetch(`/api/source/${id}`, {
				method: "DELETE",
			}).then((res) => res.json()),
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ["sources"] });
		},
	});

	const columns = useMemo(
		() => [
			columnHelper.accessor("id", {
				cell: (info) => info.getValue(),
				header: "ID",
			}),
			columnHelper.accessor("url", {
				cell: (info) => (
					<span className="break-words">
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
			columnHelper.accessor("last_checked", {
				cell: (info) => (
					<span className="break-words">{formatDate(info.getValue())}</span>
				),
				header: "Last Checked",
			}),
			columnHelper.display({
				id: "delete",
				cell: (props) => (
					<>
						<button
							disabled={loading}
							className="bg-green-700 px-4 py-3 uppercase font-bold rounded mr-2 disabled:bg-gray-700"
							onClick={async (e) => {
								e.preventDefault();

								setLoading(true);
								try {
									await enableSource.mutateAsync({ 
										id: props.row.original.id,
										enable: !props.row.original.enabled
									});
								} catch {
									console.log("deletin source failed")
								}
								setLoading(false);
							}}
						>
							{props.row.original.enabled ? "DIS" : "ENA"}
						</button>
						<button
							disabled={loading}
							className="bg-green-700 px-4 py-3 uppercase font-bold rounded disabled:bg-gray-700"
							onClick={async (e) => {
								e.preventDefault();

								setLoading(true);
								try {
									await deleteSource.mutateAsync(props.row.original.id);
								} catch {
									console.log("deletin source failed")
								}
								setLoading(false);
							}}
						>
							DEL
						</button>
					</>
				),
			}),
		],
		[deleteSource, enableSource, loading]
	);

	const table = useReactTable({
		columns,
		data: sources.data ?? [],
		getCoreRowModel: getCoreRowModel(),
	});

	return (
		<>
			<div className="max-w-lg mx-auto px-4 mb-8">
				<h3 className="text-2xl font-bold mb-4">Add Source</h3>
				<form
					onSubmit={async (e) => {
						e.preventDefault();

						setLoading(true);
						try {
							await addSource.mutateAsync({ url: url.trim() });
						} catch {
							console.log("adding source failed")
						}
						setLoading(false);
					}}
				>
					<label className="block mb-4 text-xl" htmlFor="password">
						URL
					</label>
					<div className="flex">
						<input
							value={url}
							disabled={addSource.isPending || deleteSource.isPending}
							onChange={(e) => setUrl(e.target.value)}
							className="block w-full text-black mr-2 px-4 py-3 rounded"
							id="password"
							type="url"
							placeholder="Url"
						/>
						<button
							disabled={addSource.isPending || deleteSource.isPending || loading}
							className="h-full bg-green-700 px-4 py-3 uppercase font-bold rounded disabled:bg-gray-700"
							type="submit"
						>
							Add
						</button>
					</div>
				</form>
			</div>

			<div className="max-w-4xl mx-auto px-4">
				<h3 className="text-2xl font-bold mb-4">Sources</h3>
				{sources.isError ? 
					<p>There's been an error fetching sources</p>
				: sources.isPending ?
					<p>Fetching sources...</p>
				: sources.data.length == 0 ?
					<p>There are no saved sources</p>
				: (
					<TableGrid className={style.template_3}>
						<Fragment>
							{table.getHeaderGroups().map((headerGroup) => (
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
							{table.getRowModel().rows.map((row) => (
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
