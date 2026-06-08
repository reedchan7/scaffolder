import { describe, it, expect } from "vitest";
import { greet } from "./index{{import_ext}}";

describe("greet", () => {
  it("greets by name", () => {
    expect(greet("world")).toBe("Hello, world!");
  });
});
