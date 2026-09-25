import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { BrowserRouter } from "react-router-dom";
import { LoginPage } from "../LoginPage";
import type { User } from "../../types";

const mockLogin = vi.fn();
const mockBootstrap = vi.fn();
const mockLogout = vi.fn();

vi.mock("../../auth/AuthProvider", async () => {
  const actual = await vi.importActual<typeof import("../../auth/AuthProvider")>(
    "../../auth/AuthProvider",
  );
  return {
    ...actual,
    useAuth: () => ({
      user: null as User | null,
      isAuthenticated: false,
      isLoading: false,
      login: mockLogin,
      bootstrap: mockBootstrap,
      logout: mockLogout,
    }),
  };
});

describe("LoginPage", () => {
  it("calls login on submit with valid credentials", async () => {
    mockLogin.mockResolvedValueOnce(undefined);
    render(
      <BrowserRouter>
        <LoginPage />
      </BrowserRouter>,
    );

    await userEvent.type(screen.getByLabelText(/Nome utente/i), "admin");
    await userEvent.type(screen.getByLabelText(/Password/i), "password123");
    await userEvent.click(screen.getByRole("button", { name: /Accedi/i }));

    expect(mockLogin).toHaveBeenCalledWith("admin", "password123");
  });

  it("displays error message on login failure", async () => {
    mockLogin.mockRejectedValueOnce(new Error("Credenziali non valide"));
    render(
      <BrowserRouter>
        <LoginPage />
      </BrowserRouter>,
    );

    await userEvent.type(screen.getByLabelText(/Nome utente/i), "admin");
    await userEvent.type(screen.getByLabelText(/Password/i), "wrong");
    await userEvent.click(screen.getByRole("button", { name: /Accedi/i }));

    expect(await screen.findByText(/Credenziali non valide/i)).toBeInTheDocument();
  });

  it("switches to bootstrap mode", async () => {
    render(
      <BrowserRouter>
        <LoginPage />
      </BrowserRouter>,
    );

    await userEvent.click(screen.getByRole("button", { name: /Primo accesso/i }));
    expect(screen.getByRole("button", { name: /Crea account/i })).toBeInTheDocument();
  });
});
