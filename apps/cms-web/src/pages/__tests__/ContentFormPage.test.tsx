import { describe, expect, it, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { BrowserRouter } from "react-router-dom";
import { ContentFormPage } from "../ContentFormPage";

vi.mock("react-router-dom", async () => {
  const actual = await vi.importActual("react-router-dom");
  return { ...actual, useParams: () => ({ id: undefined }), useNavigate: () => vi.fn() };
});

vi.mock("../../api/api", () => ({
  createContent: vi.fn(),
  getContent: vi.fn(),
  updateContent: vi.fn(),
  listMedia: vi.fn().mockResolvedValue([]),
}));

vi.mock("../../components/MediaPicker", () => ({
  MediaPicker: ({
    selectedId,
    onSelect,
  }: {
    selectedId: string | null;
    onSelect: (id: string | null) => void;
  }) => (
    <button type="button" onClick={() => onSelect("media-1")}>
      {selectedId ?? "Seleziona media"}
    </button>
  ),
}));

import * as api from "../../api/api";

describe("ContentFormPage", () => {
  it("validates required fields and slug format", async () => {
    render(
      <BrowserRouter>
        <ContentFormPage />
      </BrowserRouter>,
    );

    fireEvent.submit(document.body.querySelector("form")!);

    expect(await screen.findByText("Il titolo è obbligatorio.")).toBeInTheDocument();
    expect(screen.getByText("Lo slug è obbligatorio.")).toBeInTheDocument();
  });

  it("submits valid form", async () => {
    vi.mocked(api.createContent).mockResolvedValueOnce({
      id: "1",
      content_type: "news",
      title: "Test",
      slug: "test",
      excerpt: null,
      body: null,
      publication_status: "draft",
      publish_from: null,
      publish_until: null,
      published_at: null,
      image_id: null,
    });

    render(
      <BrowserRouter>
        <ContentFormPage />
      </BrowserRouter>,
    );

    await userEvent.type(screen.getByLabelText("Titolo"), "Test");
    await userEvent.clear(screen.getByLabelText("Slug"));
    await userEvent.type(screen.getByLabelText("Slug"), "test");
    await userEvent.click(screen.getByRole("button", { name: /Salva/i }));

    await screen.findByText("Contenuto salvato con successo.");
  });
});
