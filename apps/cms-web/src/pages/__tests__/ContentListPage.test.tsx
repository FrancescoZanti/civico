import { describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { BrowserRouter } from "react-router-dom";
import { ContentListPage } from "../ContentListPage";
import type { Content } from "../../types";

const mockContents: Content[] = [
  {
    id: "1",
    content_type: "news",
    title: "Notizia A",
    slug: "notizia-a",
    excerpt: null,
    body: null,
    publication_status: "published",
    publish_from: null,
    publish_until: null,
    published_at: "2026-01-01T00:00:00Z",
    image_id: null,
  },
  {
    id: "2",
    content_type: "event",
    title: "Evento B",
    slug: "evento-b",
    excerpt: null,
    body: null,
    publication_status: "draft",
    publish_from: null,
    publish_until: null,
    published_at: null,
    image_id: null,
  },
];

vi.mock("../../api/api", () => ({
  listContent: vi.fn(),
  deleteContent: vi.fn(),
}));

import * as api from "../../api/api";

describe("ContentListPage", () => {
  it("renders contents and filters by status", async () => {
    vi.mocked(api.listContent).mockResolvedValueOnce(mockContents);
    render(
      <BrowserRouter>
        <ContentListPage />
      </BrowserRouter>,
    );

    expect(await screen.findByText("Notizia A")).toBeInTheDocument();
    expect(screen.getByText("Evento B")).toBeInTheDocument();

    await userEvent.selectOptions(screen.getByLabelText("Stato"), "published");
    await waitFor(() => {
      expect(screen.queryByText("Evento B")).not.toBeInTheDocument();
    });
    expect(screen.getByText("Notizia A")).toBeInTheDocument();
  });

  it("filters by search term", async () => {
    vi.mocked(api.listContent).mockResolvedValueOnce(mockContents);
    render(
      <BrowserRouter>
        <ContentListPage />
      </BrowserRouter>,
    );

    await screen.findByText("Notizia A");
    await userEvent.type(screen.getByPlaceholderText(/Titolo/i), "Evento");
    await waitFor(() => {
      expect(screen.queryByText("Notizia A")).not.toBeInTheDocument();
    });
    expect(screen.getByText("Evento B")).toBeInTheDocument();
  });
});
