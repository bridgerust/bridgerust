/**
 * @bridgerust/kabod - Extended JavaScript API
 *
 * This module extends the native bindings with additional JavaScript-level utilities.
 */

const native = require("./index.js");

// Extend Collection prototype with insertStream
const OriginalCollection = native.Collection;

/**
 * Insert points from an async iterable in batches.
 * @param {AsyncIterable<Point>} iterable - Async iterable of Point objects
 * @param {number} batchSize - Number of points per batch (default: 1000)
 * @returns {Promise<void>}
 */
OriginalCollection.prototype.insertStream = async function (
  iterable,
  batchSize = 1000
) {
  let batch = [];

  for await (const point of iterable) {
    batch.push(point);

    if (batch.length >= batchSize) {
      await this.insert(batch);
      batch = [];
    }
  }

  // Insert remaining points
  if (batch.length > 0) {
    await this.insert(batch);
  }
};

// Re-export everything
module.exports = native;
