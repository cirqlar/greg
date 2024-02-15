import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import {
  createColumnHelper,
  flexRender,
  getCoreRowModel,
  useReactTable,
} from "@tanstack/react-table";
import { Fragment, useState } from "react";
import TableGrid from "../components/table-grid";

export const Route = createFileRoute("/_layout/activity")({
  component: Activity,
});

type TActivity = {
  id: string;
  source_url: string;
  post_url: string;
  timestamp: string;
};
const columnHelper = createColumnHelper<TActivity>();
const columns = [
  columnHelper.accessor("id", {
    cell: (info) => info.getValue(),
    header: "ID",
  }),
  columnHelper.accessor("post_url", {
    cell: (info) => (
      <span className="break-words">
        <a href={info.getValue()} target="_blank" referrerPolicy="no-referrer">
          {info.getValue()}
        </a>
      </span>
    ),
    header: "Post",
  }),
  columnHelper.accessor("source_url", {
    cell: (info) => (
      <span className="break-words">
        <a href={info.getValue()} target="_blank" referrerPolicy="no-referrer">
          {info.getValue()}
        </a>
      </span>
    ),
    header: "Source",
  }),
  columnHelper.accessor("timestamp", {
    cell: (info) => <span className="break-words">{info.getValue()}</span>,
    header: "Checked At",
  }),
];

function Activity() {
  const [loading, setLoading] = useState(false);
  const [num, setNum] = useState(0);

  const queryClient = useQueryClient();
  const activity = useQuery<TActivity[]>({
    queryKey: ["activity"],
    queryFn: () => fetch("/api/activity").then((res) => res.json()),
  });

  const table = useReactTable({
    columns: columns,
    data: activity.data ?? [],
    getCoreRowModel: getCoreRowModel(),
  });

  const recheck = useMutation({
    mutationFn: () =>
      fetch("/api/recheck", {
        method: "POST",
      }).then((res) => res.json()),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["activity"] });
    },
  });
  
  const clearActivities = useMutation({
    mutationFn: (num: number) =>
      fetch(`/api/activity${num < 1 ? "" : `/${num}`}`, {
        method: "DELETE",
      }).then((res) => res.json()),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["activity"] });
    },
  });

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
      <div className="max-w-lg mx-auto px-4 mb-8">
        <h3 className="text-2xl font-bold mb-4">Clear Activity</h3>
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
          <label className="block mb-4 text-xl" htmlFor="password">
            Number to clear (empty clears all)
          </label>
          <div className="flex">
            <input
              value={num}
              disabled={loading}
              onChange={(e) => setNum(Number(e.target.value))}
              className="block w-full text-black mr-2 px-4 py-3 rounded"
              id="password"
              type="number"
              placeholder="Url"
            />
            <button
              disabled={loading}
              className="h-full bg-green-700 px-4 py-3 uppercase font-bold rounded"
              type="submit"
            >
              Clear
            </button>
          </div>
        </form>
      </div>
      <div className="max-w-4xl mx-auto px-4">
        <h3 className="text-2xl font-bold mb-4">Activity</h3>
        <TableGrid>
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
          {/* <tfoot>
            {table.getFooterGroups().map((footerGroup) => (
              <tr key={footerGroup.id}>
                {footerGroup.headers.map((header) => (
                  <th key={header.id}>
                    {header.isPlaceholder
                      ? null
                      : flexRender(
                          header.column.columnDef.footer,
                          header.getContext()
                        )}
                  </th>
                ))}
              </tr>
            ))}
          </tfoot> */}
        </TableGrid>
      </div>
    </>
  );
}
