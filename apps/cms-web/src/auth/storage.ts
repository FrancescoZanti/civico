const TOKEN_KEY = "civico_cms_token";
const ADMIN_KEY = "civico_cms_admin";

export function getToken(): string | null {
  return localStorage.getItem(TOKEN_KEY);
}

export function setToken(token: string): void {
  localStorage.setItem(TOKEN_KEY, token);
}

export function clearAuth(): void {
  localStorage.removeItem(TOKEN_KEY);
  localStorage.removeItem(ADMIN_KEY);
}

export function getAdmin(): { id: string; username: string } | null {
  const raw = localStorage.getItem(ADMIN_KEY);
  if (!raw) return null;
  try {
    return JSON.parse(raw) as { id: string; username: string };
  } catch {
    return null;
  }
}

export function setAdmin(admin: { id: string; username: string }): void {
  localStorage.setItem(ADMIN_KEY, JSON.stringify(admin));
}
