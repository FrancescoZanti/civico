import { describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { BrowserRouter } from "react-router-dom";
import { HomeEditorPage } from "../HomeEditorPage";
import type { Content, HomeItem } from "../../types";

const mockHomeItems: HomeItem[] = [
  {
    id: "1",
    title: "Eventi",
    subtitle: null,
    icon: "📅",
    image_id: null,
    position: 0,
    enabled: true,
    destination_type: "section",
    destination_section_type: "event",
    destination_content_id: null,
  },
  {
    id: "2",
    title: "Contatto",
    subtitle: null,
    icon: "📞",
    image_id: null,
    position: 1,
    enabled: false,
    destination_type: "content",
    destination_section_type: null,
    destination_content_id: "c1",
  },
];

const mockContents: Content[] = [
  {
    id: "c1",
    content_type: "contact",
    title: "Contatto Ufficio",
    slug: "contatto-ufficio",
    excerpt: null,
    body: null,
    publication_status: "published",
    publish_from: null,
    publish_until: null,
    published_at: null,
    image_id: null,
  },
];

vi.mock("../../api/api", () => ({
  listHomeItems: vi.fn(),
  listContent: vi.fn(),
  createHomeItem: vi.fn(),
  updateHomeItem: vi.fn(),
  deleteHomeItem: vi.fn(),
}));

vi.mock("../../components/MediaPicker", () => ({
  MediaPicker: ({ onSelect }: { onSelect: (id: string | null) => void }) => (
    <button type="button" onClick={() => onSelect(null)}>
      Seleziona media
    </button>
  ),
}));

import * as api from "../../api/api";

describe("HomeEditorPage", () => {
  it("renders home items and allows enabling/disabling", async () => {
    vi.mocked(api.listHomeItems).mockResolvedValueOnce(mockHomeItems);
    vi.mocked(api.listContent).mockResolvedValueOnce(mockContents);
    vi.mocked(api.updateHomeItem).mockResolvedValueOnce({
      ...mockHomeItems[1],
      enabled: true,
    });

    render(
      <BrowserRouter>
        <HomeEditorPage />
      </BrowserRouter>,
    );

    expect(await screen.findByText("Eventi")).toBeInTheDocument();
    expect(screen.getByText("Contatto")).toBeInTheDocument();

    const toggleButtons = screen.getAllByRole("button", { name: /Abilita|Disabilita/i });
    await userEvent.click(toggleButtons[1]!);

    await waitFor(() => {
      expect(api.updateHomeItem).toHaveBeenCalledWith("2", { enabled: true });
    });
  });

  it("shows destination selection based on type", async () => {
    vi.mocked(api.listHomeItems).mockResolvedValueOnce([]);
    vi.mocked(api.listContent).mockResolvedValueOnce(mockContents);

    render(
      <BrowserRouter>
        <HomeEditorPage />
      </BrowserRouter>,
    );

    await userEvent.click(await screen.findByRole("button", { name: /Nuovo pulsante/i }));
    await userEvent.type(screen.getByLabelText("Titolo"), "Pulsante");

    await userEvent.selectOptions(screen.getByLabelText("Destinazione"), "content");
    await waitFor(() => {
      expect(screen.getByLabelText("Contenuto")).toBeInTheDocument();
    });
  });
});
