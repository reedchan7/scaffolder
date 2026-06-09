import { expect, test } from "bun:test";
import { greet } from "./index{{import_ext}}";

test("greets by name", () => {
  expect(greet("world")).toBe("Hello, world!");
});
