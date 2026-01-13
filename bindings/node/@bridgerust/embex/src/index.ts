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
      parallel?: number
    ): Promise<void>;
  }
}

// Monkey patch
(Collection.prototype as any).insertStream = async function (
  this: Collection,
  points: AsyncIterable<Point>,
  parallel: number = 5
) {
  const BATCH_SIZE = 100;
  let batch: Point[] = [];

  for await (const point of points) {
    batch.push(point);
    if (batch.length >= BATCH_SIZE) {
      await this.insertBatch(batch, BATCH_SIZE, parallel);
      batch = [];
    }
  }

  if (batch.length > 0) {
    await this.insertBatch(batch, BATCH_SIZE, parallel);
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
