// @ts-check
import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";

// https://astro.build/config
export default defineConfig({
  site: "https://bridgerust.dev",
  integrations: [
    starlight({
      title: "BridgeRust",
      social: [
        {
          icon: "github",
          label: "GitHub",
          href: "https://github.com/bridgerust/bridgerust",
        },
      ],
      sidebar: [
        {
          label: "BridgeRust",
          items: [
            { label: "Introduction", link: "/bridgerust/introduction" },
            { label: "Core Concepts", link: "/bridgerust/core-concepts" },
            { label: "Bridge CLI", link: "/bridgerust/cli" },
            { label: "The Export Macro", link: "/bridgerust/macros" },
            { label: "Advanced Usage", link: "/bridgerust/advanced-usage" },
            { label: "Architecture", link: "/bridgerust/architecture" },
          ],
        },
        {
          label: "Embex",
          items: [
            { label: "Introduction", link: "/embex/introduction" },
            { label: "Quickstart", link: "/embex/quickstart" },
            { label: "Core Concepts", link: "/embex/core-concepts" },
            { label: "Providers", link: "/embex/providers" },
            { label: "API Reference", link: "/embex/api-reference" },
          ],
        },
      ],
    }),
  ],
});
