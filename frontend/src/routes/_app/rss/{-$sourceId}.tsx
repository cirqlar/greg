import { createContext, useContext, useState } from "react";
import { createFileRoute, Link } from "@tanstack/react-router";
import * as v from "valibot";
import clsx from "clsx";

import { useAddSource, useSources } from "@/query/sources";
import { useActivity } from "@/query/activity";
import type { TSource } from "@/query/types";
import { PlusIcon } from "@storybook/icons";
import { Button } from "@/components/buttons";

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

const processingContext = createContext({
	processing: false,
	setProcessing: (_processing: boolean) => {},
});

function AddSource() {
	const { demo } = Route.useSearch();
	const { processing, setProcessing } = useContext(processingContext);

	const addSource = useAddSource();

	const [url, setUrl] = useState("");
	const [urlError, setUrlError] = useState<string>();

	const loading = addSource.isPending || processing;

	return (
		<div>
			<h3 className="mb-4 text-xl font-bold">Add Source</h3>
			<form
				onSubmit={async (e) => {
					e.preventDefault();
					if (loading) return;

					setUrlError(undefined);
					setProcessing(true);
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
					setProcessing(false);
				}}
			>
				<label
					className="sr-only mb-4 block text-xl"
					htmlFor="password"
				>
					URL
				</label>
				<div className="relative flex">
					<input
						disabled={demo || loading}
						onChange={(e) => setUrl(e.target.value)}
						className="block w-full rounded-full bg-white/20 py-3 pr-13 pl-4 text-white outline-none focus-visible:border-2 focus-visible:border-white focus-visible:py-2.5 focus-visible:pr-12.5 focus-visible:pl-3.5"
						id="password"
						type="url"
						placeholder="Url"
					/>

					<Button
						Icon={PlusIcon}
						iconLabel="Add Source"
						disabled={loading}
						animate={addSource.isPending}
						error={!!urlError}
						data-isolate
						type="submit"
						className="absolute top-0 right-0"
					/>
				</div>
				{urlError && (
					<p className="pt-4 text-center text-sm text-red-500">
						{urlError}
					</p>
				)}
			</form>
		</div>
	);
}

function Source({ source }: { source?: TSource }) {
	const { sourceId } = Route.useParams();

	const classNames = clsx(
		"flex items-center justify-between border-b py-2",
		sourceId === source?.id && "bg-white/20",
	);

	const domain = source ? new URL(source.url).hostname : undefined;

	return (
		<div className={classNames}>
			<p className={sourceId === source?.id ? "font-bold" : ""}>
				{source ? domain : "All"}{" "}
				{source && (
					<a
						href={source.url}
						target="_blank"
						referrerPolicy="no-referrer"
						className="text-sm"
					>
						Link
					</a>
				)}
			</p>

			<Link
				to="/rss/{-$sourceId}"
				params={{ sourceId: source?.id }}
				className="text-sm"
			>
				View -{">"}
			</Link>
		</div>
	);
}

function SourceList() {
	const { data: sources, error, isLoading } = useSources();

	if (isLoading) {
		return <p>Loading</p>;
	}

	if (error || !sources) {
		return <p>Error loading sources</p>;
	}

	if (sources.length === 0) {
		return <p>No sources. Add one</p>;
	}

	return (
		<div className="flex flex-col gap-2">
			<Source />
			{sources.map((source) => (
				<Source source={source} key={source.id} />
			))}
		</div>
	);
}

function ActivityList() {
	const { sourceId } = Route.useParams();

	const { data: activity, error, isLoading } = useActivity(sourceId);

	if (isLoading) {
		return <p>Loading</p>;
	}

	if (error || !activity) {
		return (
			<div className="flex h-full items-center justify-center">
				<p>Error loading activity</p>
			</div>
		);
	}

	if (activity.length === 0) {
		return (
			<div className="flex h-full items-center justify-center">
				<p>No activity</p>
			</div>
		);
	}

	return activity.map((post) => <p key={post.id}>{post.post_url}</p>);
}

function RouteComponent() {
	const [processing, setProcessing] = useState(false);

	return (
		<processingContext.Provider
			value={{
				processing: processing,
				setProcessing,
			}}
		>
			<div className="flex max-h-full justify-center gap-6 px-4 pt-24">
				<div className="flex max-h-full w-80 flex-none flex-col gap-6 overflow-auto py-4">
					<AddSource />
					<SourceList />
				</div>
				<div className="w-0.5 flex-none rounded-full bg-white/20"></div>
				<div className="max-h-full w-full overflow-auto py-4">
					<ActivityList />
				</div>
			</div>
		</processingContext.Provider>
	);
}
