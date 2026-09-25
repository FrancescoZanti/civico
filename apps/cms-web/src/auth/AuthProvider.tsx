import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useState,
  type ReactNode,
} from "react";
import { useNavigate } from "react-router-dom";
import { ApiError } from "../types";
import { clearAuth, getAdmin, getToken, setAdmin, setToken } from "../auth/storage";
import * as api from "../api/api";

interface AuthContextValue {
  token: string | null;
  admin: { id: string; username: string } | null;
  isLoading: boolean;
  login: (username: string, password: string) => Promise<void>;
  logout: () => void;
  bootstrap: (username: string, password: string) => Promise<void>;
}

const AuthContext = createContext<AuthContextValue | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [token, setTokenState] = useState<string | null>(getToken());
  const [admin, setAdminState] = useState<{ id: string; username: string } | null>(
    getAdmin(),
  );
  const [isLoading, setIsLoading] = useState(false);
  const navigate = useNavigate();

  const handleAuthResponse = (response: Awaited<ReturnType<typeof api.login>>) => {
    setToken(response.token);
    setTokenState(response.token);
    setAdmin(response.admin);
    setAdminState(response.admin);
  };

  const login = async (username: string, password: string) => {
    setIsLoading(true);
    try {
      const response = await api.login(username, password);
      handleAuthResponse(response);
      navigate("/");
    } finally {
      setIsLoading(false);
    }
  };

  const bootstrap = async (username: string, password: string) => {
    setIsLoading(true);
    try {
      const response = await api.bootstrap(username, password);
      handleAuthResponse(response);
      navigate("/");
    } finally {
      setIsLoading(false);
    }
  };

  const logout = useCallback(() => {
    clearAuth();
    setTokenState(null);
    setAdminState(null);
    navigate("/login");
  }, [navigate]);

  useEffect(() => {
    const handler = (event: ErrorEvent) => {
      if (event.error instanceof ApiError && event.error.status === 401) {
        logout();
      }
    };
    window.addEventListener("error", handler);
    return () => window.removeEventListener("error", handler);
  }, [logout]);

  return (
    <AuthContext.Provider value={{ token, admin, isLoading, login, logout, bootstrap }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error("useAuth must be used within AuthProvider");
  return ctx;
}
