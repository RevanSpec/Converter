import { describe, expect, it } from "vitest";
import { morseSchedule } from "./morse";

describe("morseSchedule", () => {
  it("donne 1 unité au point, 3 au trait et 1 entre deux signes", () => {
    expect(morseSchedule(".-")).toEqual({
      beeps: [
        { start: 0, duration: 1 },
        { start: 2, duration: 3 },
      ],
      totalUnits: 5,
    });
  });

  it("sépare deux lettres de 3 unités", () => {
    expect(morseSchedule(". .").beeps[1].start).toBe(4);
  });

  it("sépare deux mots de 7 unités, pas 10 (D3)", () => {
    expect(morseSchedule(". / .").beeps[1].start).toBe(8);
    expect(morseSchedule(".\n.").beeps[1].start).toBe(8);
  });

  it("ignore les blancs répétés et les caractères qui ne sont pas des signes", () => {
    expect(morseSchedule("  .   ?  ").beeps).toEqual([{ start: 0, duration: 1 }]);
    expect(morseSchedule("   ")).toEqual({ beeps: [], totalUnits: 0 });
  });
});
