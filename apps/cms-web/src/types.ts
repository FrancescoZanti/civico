export const API_BASE_URL = import.meta.env.VITE_API_URL || "http://127.0.0.1:8080";

export const contentTypes = [
  "page",
  "news",
  "event",
  "place",
  "contact",
  "gallery",
] as const;

export type ContentType = (typeof contentTypes)[number];

export const publicationStatuses = ["draft", "published", "archived"] as const;

export type PublicationStatus = (typeof publicationStatuses)[number];

export const homeDestinationTypes = ["section", "content"] as const;

export type HomeDestinationType = (typeof homeDestinationTypes)[number];

export interface AdminProfile {
  id: string;
  username: string;
}

export interface AuthResponse {
  token: string;
  admin: AdminProfile;
}

export interface Content {
  id: string;
  content_type: ContentType;
  title: string;
  slug: string;
  excerpt: string | null;
  body: string | null;
  publication_status: PublicationStatus;
  publish_from: string | null;
  publish_until: string | null;
  published_at: string | null;
  image_id: string | null;
}

export interface ContentCreate {
  content_type: ContentType;
  title: string;
  slug: string;
  excerpt?: string | null;
  body?: string | null;
  data?: Record<string, unknown> | null;
  publication_status?: PublicationStatus | null;
  publish_from?: string | null;
  publish_until?: string | null;
  image_id?: string | null;
}

export type ContentUpdate = Partial<ContentCreate>;

export interface HomeItem {
  id: string;
  title: string;
  subtitle: string | null;
  icon: string | null;
  image_id: string | null;
  position: number;
  enabled: boolean;
  destination_type: HomeDestinationType;
  destination_section_type: ContentType | null;
  destination_content_id: string | null;
}

export interface HomeItemCreate {
  title: string;
  subtitle?: string | null;
  icon?: string | null;
  image_id?: string | null;
  position?: number;
  enabled?: boolean;
  destination_type: HomeDestinationType;
  destination_section_type?: ContentType | null;
  destination_content_id?: string | null;
}

export type HomeItemUpdate = Partial<HomeItemCreate>;

export interface Media {
  id: string;
  filename: string;
  mime_type: string;
  size_bytes: number;
  width: number | null;
  height: number | null;
  url: string;
}

export interface PlayerHomeItem {
  id: string;
  title: string;
  subtitle: string | null;
  icon: string | null;
  image_url: string | null;
  destination: PlayerDestination;
}

export type PlayerDestination =
  { type: "section"; content_type: ContentType } | { type: "content"; slug: string };

export interface ApiErrorBody {
  error: {
    code: string;
    message: string;
    details: Record<string, unknown> | null;
  };
}

export class ApiError extends Error {
  status: number;
  code: string;
  details: Record<string, unknown> | null;

  constructor(
    status: number,
    code: string,
    message: string,
    details: Record<string, unknown> | null,
  ) {
    super(message);
    this.status = status;
    this.code = code;
    this.details = details;
    this.name = "ApiError";
  }
}

export function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / k ** i).toFixed(1))} ${sizes[i]}`;
}

export function contentTypeLabel(type: ContentType): string {
  const labels: Record<ContentType, string> = {
    page: "Pagina",
    news: "Notizia",
    event: "Evento",
    place: "Luogo",
    contact: "Contatto",
    gallery: "Galleria",
  };
  return labels[type];
}

export function publicationStatusLabel(status: PublicationStatus): string {
  const labels: Record<PublicationStatus, string> = {
    draft: "Bozza",
    published: "Pubblicato",
    archived: "Archiviato",
  };
  return labels[status];
}

export function destinationTypeLabel(type: HomeDestinationType): string {
  return type === "section" ? "Sezione" : "Contenuto";
}

export function destinationLabel(item: HomeItem, contents: Content[]): string {
  if (item.destination_type === "section") {
    return item.destination_section_type
      ? `Sezione: ${contentTypeLabel(item.destination_section_type)}`
      : "Sezione";
  }
  const content = contents.find((c) => c.id === item.destination_content_id);
  return content ? `Contenuto: ${content.title}` : "Contenuto";
}
