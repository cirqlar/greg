import { defineConfig } from "vite";

import tsconfigPaths from "vite-tsconfig-paths";
import react from "@vitejs/plugin-react";
import { tanstackRouter } from "@tanstack/router-plugin/vite";
import tailwindcss from "@tailwindcss/vite";

// https://vitejs.dev/config/
export default defineConfig({
	plugins: [
		tsconfigPaths({ projects: ["./tsconfig.json"] }),
		tailwindcss(),
		tanstackRouter(),
		react({
			babel: {
				plugins: [["babel-plugin-react-compiler"]],
			},
		}),
	],
	appType: "spa",
	build: {
		outDir: "../dist",
		emptyOutDir: true,
	},
	server: {
		proxy: {
			"/api": "http://0.0.0.0:10000",
		},
	},
});
