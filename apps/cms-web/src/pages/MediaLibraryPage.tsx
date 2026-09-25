import { useEffect, useRef, useState } from "react";
import * as api from "../api/api";
import type { Media } from "../types";
import { API_BASE_URL, ApiError, formatBytes } from "../types";
import "../styles.css";

const MAX_UPLOAD_BYTES = 10 * 1024 * 1024;

export function MediaLibraryPage() {
  const [media, setMedia] = useState<Media[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [uploading, setUploading] = useState(false);
  const [deleteId, setDeleteId] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    loadMedia();
  }, []);

  async function loadMedia() {
    setLoading(true);
    try {
      const data = await api.listMedia();
      setMedia(data);
      setError(null);
    } catch {
      setError("Impossibile caricare la libreria media.");
    } finally {
      setLoading(false);
    }
  }

  const validateFile = (file: File): string | null => {
    if (!["image/jpeg", "image/png", "image/webp"].includes(file.type)) {
      return "Formato non supportato. Sono ammessi solo JPEG, PNG e WebP.";
    }
    if (file.size > MAX_UPLOAD_BYTES) {
      return "Il file supera la dimensione massima di 10 MB.";
    }
    return null;
  };

  const handleUpload = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    const validationError = validateFile(file);
    if (validationError) {
      setError(validationError);
      if (inputRef.current) inputRef.current.value = "";
      return;
    }
    setUploading(true);
    setError(null);
    try {
      await api.uploadMedia(file);
      await loadMedia();
    } catch (err) {
      if (err instanceof ApiError) setError(err.message);
      else setError("Caricamento fallito.");
    } finally {
      setUploading(false);
      if (inputRef.current) inputRef.current.value = "";
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await api.deleteMedia(id);
      setMedia((prev) => prev.filter((m) => m.id !== id));
      setDeleteId(null);
    } catch {
      setError("Impossibile eliminare il file.");
    }
  };

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
          Media
        </h1>
        <label
          className={`cms-button cms-button-primary ${uploading ? "cms-button-disabled" : ""}`}
          style={{ position: "relative" }}
        >
          {uploading ? "Caricamento..." : "Carica file"}
          <input
            ref={inputRef}
            type="file"
            accept="image/jpeg,image/png,image/webp"
            onChange={handleUpload}
            disabled={uploading}
            style={{ position: "absolute", inset: 0, opacity: 0, cursor: "pointer" }}
          />
        </label>
      </div>
      {error && <div className="cms-form-error">{error}</div>}
      <div className="cms-card">
        {loading ? (
          <p className="cms-empty">Caricamento...</p>
        ) : media.length === 0 ? (
          <p className="cms-empty">Nessun file caricato.</p>
        ) : (
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(auto-fill, minmax(180px, 1fr))",
              gap: "1rem",
            }}
          >
            {media.map((m) => (
              <div
                key={m.id}
                style={{
                  border: "1px solid #dee2e6",
                  borderRadius: "0.5rem",
                  overflow: "hidden",
                  background: "#fff",
                }}
              >
                <img
                  src={`${API_BASE_URL}${m.url}`}
                  alt={m.filename}
                  style={{ width: "100%", height: 140, objectFit: "cover" }}
                />
                <div style={{ padding: "0.75rem" }}>
                  <div
                    style={{
                      fontSize: "0.875rem",
                      fontWeight: 500,
                      overflow: "hidden",
                      textOverflow: "ellipsis",
                      whiteSpace: "nowrap",
                    }}
                  >
                    {m.filename}
                  </div>
                  <div style={{ fontSize: "0.75rem", color: "#6c757d" }}>
                    {formatBytes(m.size_bytes)} ·{" "}
                    {m.mime_type.replace("image/", "").toUpperCase()}
                  </div>
                  <div className="cms-actions" style={{ marginTop: "0.75rem" }}>
                    <a
                      href={`${API_BASE_URL}${m.url}`}
                      target="_blank"
                      rel="noreferrer"
                      className="cms-button cms-button-secondary"
                      style={{ fontSize: "0.875rem" }}
                    >
                      Apri
                    </a>
                    <button
                      type="button"
                      className="cms-button cms-button-danger"
                      style={{ fontSize: "0.875rem" }}
                      onClick={() => setDeleteId(m.id)}
                    >
                      Elimina
                    </button>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
      {deleteId && (
        <div className="cms-modal-overlay">
          <div className="cms-modal">
            <h2 className="cms-modal-title">Conferma eliminazione</h2>
            <p>Eliminare definitivamente questo file?</p>
            <div className="cms-modal-actions">
              <button
                type="button"
                className="cms-button cms-button-secondary"
                onClick={() => setDeleteId(null)}
              >
                Annulla
              </button>
              <button
                type="button"
                className="cms-button cms-button-danger"
                onClick={() => handleDelete(deleteId)}
              >
                Elimina
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
