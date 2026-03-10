export function handleFetchResponse(errorText: string = "Error fetching") {
	return async (res: Response) => {
		if (res.ok) {
			return res.json();
		} else {
			let err;
			try {
				err = await res.json();
			} catch {
				err = "Non-json return from response";
			}
			console.log(errorText, err);
			throw new Error(err);
		}
	};
}
