import { execFileSync } from "node:child_process";
import { copyFileSync } from "node:fs";
import { resolve } from "node:path";

const output = execFileSync("cargo", [
  "build",
  "--manifest-path", "../Cargo.toml",
  "--locked",
  "--release",
  "--features", "nodejs",
  "--message-format=json-render-diagnostics",
], { encoding: "utf8", stdio: ["ignore", "pipe", "inherit"] });

const artifact = output.trim().split("\n").map((line) => JSON.parse(line))
  .find((message) => message.reason === "compiler-artifact" &&
    message.target.name === "bridge_date_parity" &&
    message.target.crate_types.includes("cdylib"));
const library = artifact?.filenames.find((name) => /\.(dll|dylib|so)$/.test(name));
if (!library) throw new Error("Cargo did not produce the date parity native binding");
copyFileSync(library, resolve("index.node"));
