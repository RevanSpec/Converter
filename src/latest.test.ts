import { describe, expect, it } from "vitest";
import { LatestRequest } from "./latest";

describe("LatestRequest", () => {
  it("rend périmée une requête dès qu'une autre démarre", () => {
    const requests = new LatestRequest();
    const first = requests.begin();
    const second = requests.begin();
    expect(first()).toBe(false);
    expect(second()).toBe(true);
  });

  it("rend périmées toutes les requêtes en cours", () => {
    const requests = new LatestRequest();
    const pending = requests.begin();
    requests.invalidate();
    expect(pending()).toBe(false);
  });
});
