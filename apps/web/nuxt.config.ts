import tailwindcss from "@tailwindcss/vite";

export default defineNuxtConfig({
	compatibilityDate: "2025-07-15",
	devtools: { enabled: true },
	css: ["~/assets/main.css"],
	vite: {
		plugins: [tailwindcss()],
	},
	app: {
		head: {
			htmlAttrs: { lang: "es" },
			title: "Lexicon | Reto semantico diario",
			meta: [
				{
					name: "description",
					content: "El juego diario para encontrar el nexo entre palabras.",
				},
				{ name: "theme-color", content: "#101311" },
				{ name: "color-scheme", content: "dark" },
			],
		},
	},
	runtimeConfig: {
		public: {
			apiBase: process.env.NUXT_PUBLIC_API_BASE ?? "http://127.0.0.1:3001",
		},
	},
});
