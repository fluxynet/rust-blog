import { defineConfig } from "orval";

export default defineConfig({
  api: {
    input: "../../openapi.json",
    output: {
      mode: "split",
      target: "src/api",
      client: "axios"
    }
  }
})
