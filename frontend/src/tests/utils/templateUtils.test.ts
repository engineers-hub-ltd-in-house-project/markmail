import { describe, it, expect } from "vitest";

// 変数パース機能のテスト関数
function parseVariables(text: string): Record<string, string> {
  const vars: Record<string, string> = {};
  if (!text.trim()) return vars;

  try {
    const lines = text.split("\n");
    lines.forEach((line) => {
      const trimmed = line.trim();
      if (trimmed && trimmed.includes("=")) {
        const [key, ...valueParts] = trimmed.split("=");
        const value = valueParts.join("=").trim();
        if (key.trim() && value) {
          vars[key.trim()] = value;
        }
      }
    });
  } catch (err) {
    console.error("Variables parsing error:", err);
  }

  return vars;
}

// 変数からテキスト変換関数
function variablesToText(variables: Record<string, string> = {}): string {
  return Object.entries(variables)
    .map(([key, value]) => `${key}=${value}`)
    .join("\n");
}

describe("Template Utilities", () => {
  describe("parseVariables", () => {
    it("should correctly parse variable format", () => {
      const input =
        "name=John Doe\ncompany=Example Corp\nemail=john@example.com";
      const expected = {
        name: "John Doe",
        company: "Example Corp",
        email: "john@example.com",
      };

      expect(parseVariables(input)).toEqual(expected);
    });

    it("should handle empty input", () => {
      expect(parseVariables("")).toEqual({});
      expect(parseVariables("   ")).toEqual({});
    });

    it("should handle values containing equals sign", () => {
      const input = "formula=2+2=4\nurl=https://example.com?param=value";
      const expected = {
        formula: "2+2=4",
        url: "https://example.com?param=value",
      };

      expect(parseVariables(input)).toEqual(expected);
    });

    it("should ignore malformatted lines", () => {
      const input = "name=John\nmalformatted-line\ncompany=Example";
      const expected = {
        name: "John",
        company: "Example",
      };

      expect(parseVariables(input)).toEqual(expected);
    });

    it("should trim whitespace from keys and values", () => {
      const input = "  name  =  John Doe  \n  company  =  Example Corp  ";
      const expected = {
        name: "John Doe",
        company: "Example Corp",
      };

      expect(parseVariables(input)).toEqual(expected);
    });

    it("should ignore lines with empty keys or values", () => {
      const input = "name=John\n=Empty Key\nEmptyValue=\n=";
      const expected = {
        name: "John",
      };

      expect(parseVariables(input)).toEqual(expected);
    });
  });

  describe("variablesToText", () => {
    it("should convert variables object to text format", () => {
      const variables = {
        name: "John Doe",
        company: "Example Corp",
        email: "john@example.com",
      };

      const result = variablesToText(variables);

      // Different line orders are possible, so we'll check each line
      expect(result.split("\n")).toHaveLength(3);
      expect(result).toContain("name=John Doe");
      expect(result).toContain("company=Example Corp");
      expect(result).toContain("email=john@example.com");
    });

    it("should handle empty object", () => {
      expect(variablesToText({})).toBe("");
    });

    it("should handle undefined input", () => {
      expect(variablesToText(undefined)).toBe("");
    });
  });
});
