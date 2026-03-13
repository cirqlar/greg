import { defineConfig } from "vite";

import tailwindcss from "@tailwindcss/vite";
import { tanstackRouter } from "@tanstack/router-plugin/vite";
import react, { reactCompilerPreset } from "@vitejs/plugin-react";
import babel from "@rolldown/plugin-babel";

// https://vitejs.dev/config/
export default defineConfig({
	resolve: { tsconfigPaths: true },
	plugins: [
		tailwindcss(),
		tanstackRouter(),
		react(),
		babel({
			presets: [reactCompilerPreset()],
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
