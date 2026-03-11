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
	change_count: number | null;
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

export type TTabChange = {
	type: "tab_added" | "tab_removed";
	tab_id: string;
	tab_db_id: number;
	tab_name: string;
	tab_slug: string;
};

export type TCardAdded = {
	type: "card_added";
	current_card_id: string;
	current_card_db_id: number;
	current_card_name: string;
	current_card_description: string;
	current_card_image_url?: string;
	current_card_slug: string;
	card_tab_name: string;
};

export type TCardRemoved = {
	type: "card_removed";
	previous_card_id: string;
	previous_card_db_id: number;
	previous_card_name: string;
	previous_card_description: string;
	previous_card_image_url?: string;
	previous_card_slug: string;
	card_tab_name: string;
};

export type TCardModified = {
	type: "card_modified";
	previous_card_id: string;
	previous_card_db_id: number;
	previous_card_name: string;
	previous_card_description: string;
	previous_card_image_url?: string;
	previous_card_slug: string;
	current_card_id: string;
	current_card_db_id: number;
	current_card_name: string;
	current_card_description: string;
	current_card_image_url?: string;
	current_card_slug: string;
	card_tab_name: string;
};

export type TRoadmapChange = {
	id: number;
} & (TTabChange | TCardAdded | TCardRemoved | TCardModified);
