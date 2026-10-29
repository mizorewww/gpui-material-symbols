import { bindings, defineConfig } from "cf/config";

export default defineConfig({
	worker: {
		name: "m3symbols",
		compatibilityDate: "2026-10-01",
		entrypoint: "worker.js",
		assets: {
			runWorkerFirst: [
				"/m/*",
			],
		},
		domains: [
			"m3symbols.mizore.blog",
		],
		env: {
			ASSETS: bindings.assets(),
		},
	},
});
