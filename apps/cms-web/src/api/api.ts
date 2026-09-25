import { apiFetch } from "./client";
import type {
  AuthResponse,
  Content,
  ContentCreate,
  ContentUpdate,
  HomeItem,
  HomeItemCreate,
  HomeItemUpdate,
  Media,
  PlayerHomeItem,
} from "../types";

export function bootstrap(username: string, password: string): Promise<AuthResponse> {
  return apiFetch<AuthResponse>("/api/v1/admin/auth/bootstrap", {
    method: "POST",
    body: { username, password },
  });
}

export function login(username: string, password: string): Promise<AuthResponse> {
  return apiFetch<AuthResponse>("/api/v1/admin/auth/login", {
    method: "POST",
    body: { username, password },
  });
}

export function listContent(): Promise<Content[]> {
  return apiFetch<Content[]>("/api/v1/admin/content");
}

export function getContent(id: string): Promise<Content> {
  return apiFetch<Content>(`/api/v1/admin/content/${id}`);
}

export function createContent(body: ContentCreate): Promise<Content> {
  return apiFetch<Content>("/api/v1/admin/content", {
    method: "POST",
    body,
  });
}

export function updateContent(id: string, body: ContentUpdate): Promise<Content> {
  return apiFetch<Content>(`/api/v1/admin/content/${id}`, {
    method: "PUT",
    body,
  });
}

export function deleteContent(id: string): Promise<void> {
  return apiFetch<void>(`/api/v1/admin/content/${id}`, {
    method: "DELETE",
  });
}

export function listHomeItems(): Promise<HomeItem[]> {
  return apiFetch<HomeItem[]>("/api/v1/admin/home-items");
}

export function getHomeItem(id: string): Promise<HomeItem> {
  return apiFetch<HomeItem>(`/api/v1/admin/home-items/${id}`);
}

export function createHomeItem(body: HomeItemCreate): Promise<HomeItem> {
  return apiFetch<HomeItem>("/api/v1/admin/home-items", {
    method: "POST",
    body,
  });
}

export function updateHomeItem(id: string, body: HomeItemUpdate): Promise<HomeItem> {
  return apiFetch<HomeItem>(`/api/v1/admin/home-items/${id}`, {
    method: "PUT",
    body,
  });
}

export function deleteHomeItem(id: string): Promise<void> {
  return apiFetch<void>(`/api/v1/admin/home-items/${id}`, {
    method: "DELETE",
  });
}

export function listMedia(): Promise<Media[]> {
  return apiFetch<Media[]>("/api/v1/admin/media");
}

export function uploadMedia(file: File): Promise<Media> {
  const formData = new FormData();
  formData.append("file", file);
  return apiFetch<Media>("/api/v1/admin/media", {
    method: "POST",
    body: formData,
  });
}

export function deleteMedia(id: string): Promise<void> {
  return apiFetch<void>(`/api/v1/admin/media/${id}`, {
    method: "DELETE",
  });
}

export function getPlayerHome(): Promise<PlayerHomeItem[]> {
  return apiFetch<PlayerHomeItem[]>("/api/v1/player/home");
}
