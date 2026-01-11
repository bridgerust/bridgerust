// @ts-check
import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";

// https://astro.build/config
export default defineConfig({
  site: "https://bridgerust.dev",
  integrations: [
    starlight({
      title: "BridgeRust",
      components: {
        Head: "./src/components/AnalyticsHead.astro",
      },
      social: [
        {
          icon: "github",
          label: "GitHub",
          href: "https://github.com/bridgerust/bridgerust",
        },
        {
          icon: "discord",
          label: "Discord",
          href: "https://discord.gg/ZvNAeaWN",
        },
        {
          icon: "reddit",
          label: "Reddit",
          href: "https://www.reddit.com/r/embex/",
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
            { label: "Installation", link: "/embex/installation" },
            { label: "Quickstart", link: "/embex/quickstart" },
            { label: "Tutorial: Chatbot", link: "/embex/tutorial" },
            { label: "Core Concepts", link: "/embex/core-concepts" },
            { label: "Providers", link: "/embex/providers" },
            { label: "Migrations", link: "/embex/migrations" },
            { label: "Benchmarks", link: "/embex/benchmarks" },
            {
              label: "API Reference",
              items: [
                { label: "Python API", link: "/embex/api/python" },
                { label: "Node.js API", link: "/embex/api/nodejs" },
              ],
            },
          ],
        },
      ],
    }),
  ],
});
