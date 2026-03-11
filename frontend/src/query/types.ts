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

export type TRoadmapActivity = {
	id: number;
	timestamp: string;
};

export type TWatchedTab = {
	id: number;
	tab_id: string;
	timestamp: string;
};

export type TRTab = {
	id: string;
	name: string;
	slug: string;
	db_id: number;
};
