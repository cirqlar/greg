import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { useState } from "react";

export const Route = createFileRoute("/_layout/activity")({
  component: Activity,
});

function Activity() {
  const [loading, setLoading] = useState(false);

  const queryClient = useQueryClient();
  const activity = useQuery<
    { id: string; source_id: string; post_url: string; timestamp: string }[]
  >({
    queryKey: ["activity"],
    queryFn: () => fetch("/api/activity").then((res) => res.json()),
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
              await recheck.mutate();
            } catch {
              console.log("Recheck failed");
            }
            setLoading(false);
          }}
        >
          Recheck
        </button>
      </div>
      <div className="max-w-lg mx-auto px-4">
        <h3 className="text-2xl font-bold mb-4">Activity</h3>
        <ul>
          {activity.data?.map((event) => (
            <li key={event.id}>
              {event.post_url} from {event.source_id} at {event.timestamp}
            </li>
          ))}
        </ul>
      </div>
    </>
  );
}
