import { create } from "zustand";

type ProcessingStoreState = {
	processing: boolean;
};

export const useProcessing = create<ProcessingStoreState>()(() => ({
	processing: false,
}));

export function updateProcessing(n: boolean) {
	useProcessing.setState({ processing: n });
}
