import {
  Collection,
  EmbexClient,
  QueryBuilder,
  SearchBuilder,
  DataMigrator,
  ScrollResponse,
  MigrationResult,
  // @ts-ignore
  cli,
  Point,
} from "../native";

declare module "../native" {
  interface Collection {
    insertStream(
      points: AsyncIterable<Point>,
      batchSize?: number,
      parallel?: number
    ): Promise<void>;
  }
}

const nativeInsertStream = (Collection.prototype as any).insertStream;

// API-level adapter: accepts AsyncIterable while delegating to native ReadableStream path.
(Collection.prototype as any).insertStream = async function (
  this: Collection,
  points: AsyncIterable<Point>,
  batchSize: number = 1000,
  parallel: number = 5
) {
  if (batchSize <= 0) {
    throw new Error("batchSize must be greater than 0");
  }

  let batch: Point[] = [];
  for await (const point of points) {
    batch.push(point);
    if (batch.length >= batchSize) {
      if (typeof nativeInsertStream === "function") {
        await nativeInsertStream.call(this, batch, batchSize, parallel);
      } else {
        await this.insertBatch(batch, batchSize, parallel);
      }
      batch = [];
    }
  }

  if (batch.length > 0) {
    if (typeof nativeInsertStream === "function") {
      await nativeInsertStream.call(this, batch, batchSize, parallel);
    } else {
      await this.insertBatch(batch, batchSize, parallel);
    }
  }
};

export {
  Collection,
  EmbexClient,
  QueryBuilder,
  SearchBuilder,
  DataMigrator,
  ScrollResponse,
  MigrationResult,
  cli,
  Point,
};
