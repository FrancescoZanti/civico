import { useEffect, useState } from "react";
import * as api from "../api/api";
import type { PlayerHomeItem } from "../types";
import { API_BASE_URL, contentTypeLabel } from "../types";
import "../styles.css";

const PLAYER_URL = import.meta.env.VITE_PLAYER_URL || "http://127.0.0.1:8081";

export function PreviewPage() {
  const [items, setItems] = useState<PlayerHomeItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    async function load() {
      try {
        const data = await api.getPlayerHome();
        if (!cancelled) setItems(data);
      } catch {
        if (!cancelled)
          setError(
            "Impossibile caricare l'anteprima. Verifica che i contenuti siano pubblicati.",
          );
      } finally {
        if (!cancelled) setLoading(false);
      }
    }
    load();
    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <div>
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          marginBottom: "1.5rem",
        }}
      >
        <h1 className="cms-page-title" style={{ marginBottom: 0 }}>
          Anteprima kiosk
        </h1>
        <a
          href={PLAYER_URL}
          target="_blank"
          rel="noreferrer"
          className="cms-button cms-button-secondary"
        >
          Apri a schermo intero
        </a>
      </div>
      {error && <div className="cms-form-error">{error}</div>}
      <div
        className="cms-card"
        style={{ display: "flex", justifyContent: "center", padding: "2rem" }}
      >
        <div
          style={{
            width: 360,
            height: 640,
            border: "12px solid #212529",
            borderRadius: "2rem",
            backgroundColor: "#f8f9fa",
            overflow: "hidden",
            display: "flex",
            flexDirection: "column",
          }}
        >
          <div
            style={{
              backgroundColor: "#0d6efd",
              color: "#fff",
              padding: "1rem",
              textAlign: "center",
              fontWeight: 600,
            }}
          >
            Civico
          </div>
          <div style={{ flex: 1, padding: "1rem", overflowY: "auto" }}>
            {loading ? (
              <p className="cms-empty">Caricamento...</p>
            ) : items.length === 0 ? (
              <p className="cms-empty">
                Nessun pulsante visibile. Pubblica e abilita almeno un elemento HOME.
              </p>
            ) : (
              <div style={{ display: "flex", flexDirection: "column", gap: "0.75rem" }}>
                {items.map((item) => (
                  <div
                    key={item.id}
                    style={{
                      backgroundColor: "#fff",
                      border: "1px solid #dee2e6",
                      borderRadius: "0.5rem",
                      padding: "1rem",
                      display: "flex",
                      alignItems: "center",
                      gap: "0.75rem",
                    }}
                  >
                    {item.image_url ? (
                      <img
                        src={`${API_BASE_URL}${item.image_url}`}
                        alt=""
                        style={{
                          width: 48,
                          height: 48,
                          objectFit: "cover",
                          borderRadius: "0.25rem",
                        }}
                      />
                    ) : (
                      <div style={{ fontSize: "1.5rem" }}>{item.icon || "🔘"}</div>
                    )}
                    <div>
                      <div style={{ fontWeight: 600 }}>{item.title}</div>
                      <div style={{ fontSize: "0.75rem", color: "#6c757d" }}>
                        {item.destination.type === "section"
                          ? contentTypeLabel(item.destination.content_type)
                          : `Contenuto: ${item.destination.slug}`}
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
