import { describe, expect, it, vi } from "vitest";
import { apiFetch } from "../client";
import { ApiError } from "../../types";
import * as storage from "../../auth/storage";

vi.mock("../../auth/storage", () => ({
  getToken: vi.fn(),
}));

global.fetch = vi.fn();

describe("apiFetch", () => {
  it("includes bearer token when available", async () => {
    vi.mocked(storage.getToken).mockReturnValue("test-token");
    vi.mocked(fetch).mockResolvedValueOnce(
      new Response(JSON.stringify({ ok: true }), { status: 200 }),
    );

    await apiFetch("/api/v1/admin/content");

    expect(fetch).toHaveBeenCalledWith(
      expect.any(String),
      expect.objectContaining({
        headers: expect.any(Headers),
      }),
    );
    const callArgs = vi.mocked(fetch).mock.calls[0];
    const headers = (callArgs[1] as RequestInit).headers as Headers;
    expect(headers.get("Authorization")).toBe("Bearer test-token");
  });

  it("throws ApiError on 401", async () => {
    vi.mocked(storage.getToken).mockReturnValue("expired");
    vi.mocked(fetch).mockResolvedValueOnce(
      new Response(
        JSON.stringify({
          error: { code: "unauthenticated", message: "Token non valido", details: null },
        }),
        { status: 401 },
      ),
    );

    await expect(apiFetch("/api/v1/admin/content")).rejects.toBeInstanceOf(ApiError);
  });

  it("throws network error when fetch fails", async () => {
    vi.mocked(storage.getToken).mockReturnValue(null);
    vi.mocked(fetch).mockRejectedValueOnce(new TypeError("Failed to fetch"));

    await expect(apiFetch("/api/v1/admin/content")).rejects.toBeInstanceOf(ApiError);
  });
});
