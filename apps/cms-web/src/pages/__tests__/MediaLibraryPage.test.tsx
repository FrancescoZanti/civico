import { describe, expect, it, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { BrowserRouter } from "react-router-dom";
import { MediaLibraryPage } from "../MediaLibraryPage";
import type { Media } from "../../types";

const mockMedia: Media[] = [
  {
    id: "m1",
    filename: "foto.jpg",
    mime_type: "image/jpeg",
    size_bytes: 1024,
    width: null,
    height: null,
    url: "/api/v1/media/m1/file",
  },
];

vi.mock("../../api/api", () => ({
  listMedia: vi.fn(),
  uploadMedia: vi.fn(),
  deleteMedia: vi.fn(),
}));

import * as api from "../../api/api";

describe("MediaLibraryPage", () => {
  it("renders media list", async () => {
    vi.mocked(api.listMedia).mockResolvedValueOnce(mockMedia);
    render(
      <BrowserRouter>
        <MediaLibraryPage />
      </BrowserRouter>,
    );

    expect(await screen.findByText("foto.jpg")).toBeInTheDocument();
  });

  it("rejects unsupported file types", async () => {
    vi.mocked(api.listMedia).mockResolvedValueOnce([]);
    render(
      <BrowserRouter>
        <MediaLibraryPage />
      </BrowserRouter>,
    );

    const input = screen.getByLabelText("Carica file", { selector: "input" });
    const file = new File(["<svg></svg>"], "evil.svg", { type: "image/svg+xml" });
    fireEvent.change(input, { target: { files: [file] } });

    expect(
      screen.getByText("Formato non supportato. Sono ammessi solo JPEG, PNG e WebP."),
    ).toBeInTheDocument();
    expect(api.uploadMedia).not.toHaveBeenCalled();
  });
});
