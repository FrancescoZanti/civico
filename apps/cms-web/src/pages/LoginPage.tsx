import { useState } from "react";
import { useAuth } from "../auth/AuthProvider";
import { ApiError } from "../types";
import "../styles.css";

export function LoginPage() {
  const { login, bootstrap, isLoading } = useAuth();
  const [isBootstrap, setIsBootstrap] = useState(false);
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    try {
      if (isBootstrap) {
        await bootstrap(username, password);
      } else {
        await login(username, password);
      }
    } catch (err) {
      if (err instanceof ApiError) {
        setError(err.message);
      } else if (err instanceof Error) {
        setError(err.message);
      } else {
        setError("Si è verificato un errore imprevisto.");
      }
    }
  };

  return (
    <div className="cms-login-page">
      <div className="cms-login-card">
        <h1 className="cms-login-title">Civico CMS</h1>
        <p className="cms-login-subtitle">
          {isBootstrap
            ? "Crea il primo account amministratore"
            : "Accedi all'area di amministrazione"}
        </p>
        {error && <div className="cms-form-error">{error}</div>}
        <form onSubmit={handleSubmit}>
          <div className="cms-form-group">
            <label htmlFor="username" className="cms-label">
              Nome utente
            </label>
            <input
              id="username"
              type="text"
              className="cms-input"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              required
              autoFocus
            />
          </div>
          <div className="cms-form-group">
            <label htmlFor="password" className="cms-label">
              Password
            </label>
            <input
              id="password"
              type="password"
              className="cms-input"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              required
              minLength={8}
            />
          </div>
          <button
            type="submit"
            className="cms-button cms-button-primary"
            disabled={isLoading}
            style={{ width: "100%" }}
          >
            {isLoading ? "Accesso in corso..." : isBootstrap ? "Crea account" : "Accedi"}
          </button>
        </form>
        <div style={{ marginTop: "1rem", textAlign: "center" }}>
          <button
            type="button"
            className="cms-button cms-button-ghost"
            onClick={() => setIsBootstrap((v) => !v)}
            style={{ fontSize: "0.875rem" }}
          >
            {isBootstrap ? "Ho già un account" : "Primo accesso? Crea admin"}
          </button>
        </div>
      </div>
    </div>
  );
}
