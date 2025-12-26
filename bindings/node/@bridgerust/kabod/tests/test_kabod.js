const { KabodClient } = require("../index.js");

async function test() {
  console.log("Testing Kabod Node.js bindings...");

  // Mock server or expect error
  try {
    const client = new KabodClient("qdrant", "http://localhost:6333");
    console.log("Client created successfully");

    const collection = client.collection("test-collection");
    console.log("Collection handle created: " + (collection ? "yes" : "no"));

    // We can't really run methods without a backend, but we can verify existence
    if (typeof collection.insert === "function") {
      console.log("collection.insert is a function");
    }

    if (typeof collection.search === "function") {
      console.log("collection.search is a function");
    }
  } catch (e) {
    console.error("Error:", e);
    process.exit(1);
  }
}

test();
