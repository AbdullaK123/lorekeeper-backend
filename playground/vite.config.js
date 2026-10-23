import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// /api/* → backend on :8000. Same origin for the browser, so the session cookie just works.
export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,
    proxy: {
      "/api": {
        target: "http://localhost:8000",
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api/, ""),
      },
    },
  },
});
