import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { ProtectedRoute, PublicOnlyRoute } from "../RouteGuards";
import { useAuth } from "../AuthProvider";

vi.mock("../AuthProvider", () => ({
  useAuth: vi.fn(),
}));

function TestComponent({ text }: { text: string }) {
  return <div>{text}</div>;
}

describe("RouteGuards", () => {
  it("ProtectedRoute redirects to /login when not authenticated", () => {
    vi.mocked(useAuth).mockReturnValue({
      token: null,
      admin: null,
      isLoading: false,
      login: vi.fn(),
      logout: vi.fn(),
      bootstrap: vi.fn(),
    });
    render(
      <MemoryRouter initialEntries={["/dashboard"]}>
        <Routes>
          <Route path="/login" element={<TestComponent text="Login" />} />
          <Route
            path="/dashboard"
            element={
              <ProtectedRoute>
                <TestComponent text="Dashboard" />
              </ProtectedRoute>
            }
          />
        </Routes>
      </MemoryRouter>,
    );
    expect(screen.getByText("Login")).toBeInTheDocument();
  });

  it("ProtectedRoute renders children when authenticated", () => {
    vi.mocked(useAuth).mockReturnValue({
      token: "abc",
      admin: { id: "1", username: "admin" },
      isLoading: false,
      login: vi.fn(),
      logout: vi.fn(),
      bootstrap: vi.fn(),
    });
    render(
      <MemoryRouter initialEntries={["/dashboard"]}>
        <Routes>
          <Route path="/login" element={<TestComponent text="Login" />} />
          <Route
            path="/dashboard"
            element={
              <ProtectedRoute>
                <TestComponent text="Dashboard" />
              </ProtectedRoute>
            }
          />
        </Routes>
      </MemoryRouter>,
    );
    expect(screen.getByText("Dashboard")).toBeInTheDocument();
  });

  it("PublicOnlyRoute redirects to / when authenticated", () => {
    vi.mocked(useAuth).mockReturnValue({
      token: "abc",
      admin: { id: "1", username: "admin" },
      isLoading: false,
      login: vi.fn(),
      logout: vi.fn(),
      bootstrap: vi.fn(),
    });
    render(
      <MemoryRouter initialEntries={["/login"]}>
        <Routes>
          <Route path="/" element={<TestComponent text="Home" />} />
          <Route
            path="/login"
            element={
              <PublicOnlyRoute>
                <TestComponent text="Login" />
              </PublicOnlyRoute>
            }
          />
        </Routes>
      </MemoryRouter>,
    );
    expect(screen.getByText("Home")).toBeInTheDocument();
  });
});
