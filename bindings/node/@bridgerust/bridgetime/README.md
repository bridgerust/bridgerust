# @bridgerust/bridgetime

BridgeTime is a Rust-powered datetime toolkit with a Day.js/Moment-style API for Node.js.

## Install

```bash
npm install @bridgerust/bridgetime
```

## Quickstart

```ts
import { BridgeTime, supportedUnits } from "@bridgerust/bridgetime";

const now = BridgeTime.now("UTC");
console.log(now.toIso());

const ny = now.toTimezone("America/New_York");
console.log(ny.format("YYYY-MM-DD HH:mm:ss"));

const next = now.add(1, "month").startOf("day");
console.log(next.toIso());

console.log(supportedUnits());
```

## Build Locally

```bash
cd bindings/node/@bridgerust/bridgetime
npm install
npm run build
```
