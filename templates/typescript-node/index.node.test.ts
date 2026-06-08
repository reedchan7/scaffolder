import { test } from "node:test";
import assert from "node:assert/strict";
import { greet } from "./index{{import_ext}}";

test("greets by name", () => {
  assert.equal(greet("world"), "Hello, world!");
});
