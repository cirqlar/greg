import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/_layout/sources")({
  component: Sources,
});

function Sources() {
  const [url, setUrl] = useState("");
  const queryClient = useQueryClient();
  const sources = useQuery<{ id: string; url: string; last_checked: string }[]>(
    {
      queryKey: ["sources"],
      queryFn: () => fetch("/api/sources").then((res) => res.json()),
    }
  );

  const addSource = useMutation({
    mutationFn: (source: { url: string }) =>
      fetch("/api/source/new", {
        method: "POST",
		headers: {
			"Content-Type": 'application/json',
		},
        body: JSON.stringify(source),
      }).then((res) => res.json()),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["sources"] });
    },
  });

  return (
    <>
      <div className="max-w-lg mx-auto px-4 mb-8">
        <h3 className="text-2xl font-bold mb-4">Add Source</h3>
        <form
          onSubmit={async (e) => {
            e.preventDefault();

            await addSource.mutate({ url });
          }}
        >
          <label className="block mb-4 text-xl" htmlFor="password">
            URL
          </label>
          <div className="flex">
            <input
              value={url}
              disabled={addSource.isPending}
              onChange={(e) => setUrl(e.target.value)}
              className="block w-full text-black mr-2 px-4 py-3 rounded"
              id="password"
              type="url"
              placeholder="Url"
            />
            <button
              disabled={addSource.isPending}
              className="h-full bg-green-700 px-4 py-3 uppercase font-bold rounded"
              type="submit"
            >
              Add
            </button>
          </div>
        </form>
      </div>

      <div className="max-w-lg mx-auto px-4">
        <h3 className="text-2xl font-bold mb-4">Sources</h3>
        <ul>
          {sources.data?.map((source) => <li key={source.id}>{source.url}</li>)}
        </ul>
      </div>
    </>
  );
}
