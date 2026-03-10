export type TSource = {
	id: number;
	url: string;
	last_checked: string;
	enabled: boolean;
};

export type TActivity = {
	id: string;
	source_url: string;
	post_url: string;
	timestamp: string;
};
