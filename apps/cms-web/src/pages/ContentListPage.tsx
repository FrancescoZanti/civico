import { useEffect, useMemo, useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import * as api from "../api/api";
import type { Content, ContentType, PublicationStatus } from "../types";
import {
  contentTypeLabel,
  contentTypes,
  publicationStatusLabel,
  publicationStatuses,
} from "../types";
import "../styles.css";

export function ContentListPage() {
  const [contents, setContents] = useState<Content[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [typeFilter, setTypeFilter] = useState<ContentType | "">("");
  const [statusFilter, setStatusFilter] = useState<PublicationStatus | "">("");
  const [search, setSearch] = useState("");
  const [deleteId, setDeleteId] = useState<string | null>(null);
  const navigate = useNavigate();

  useEffect(() => {
    let cancelled = false;
    async function load() {
      try {
        const data = await api.listContent();
        if (!cancelled) setContents(data);
      } catch {
        if (!cancelled) setError("Impossibile caricare i contenuti.");
      } finally {
        if (!cancelled) setLoading(false);
      }
    }
    load();
    return () => {
      cancelled = true;
    };
  }, []);

  const filtered = useMemo(() => {
    return contents
      .filter((c) => (typeFilter ? c.content_type === typeFilter : true))
      .filter((c) => (statusFilter ? c.publication_status === statusFilter : true))
      .filter((c) =>
        search ? c.title.toLowerCase().includes(search.toLowerCase()) : true,
      );
  }, [contents, typeFilter, statusFilter, search]);

  const handleDelete = async (id: string) => {
    try {
      await api.deleteContent(id);
      setContents((prev) => prev.filter((c) => c.id !== id));
      setDeleteId(null);
    } catch {
      setError("Impossibile eliminare il contenuto.");
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
          Contenuti
        </h1>
        <Link to="/content/new" className="cms-button cms-button-primary">
          Nuovo contenuto
        </Link>
      </div>
      {error && <div className="cms-form-error">{error}</div>}
      <div
        className="cms-card"
        style={{ display: "flex", gap: "1rem", flexWrap: "wrap", alignItems: "end" }}
      >
        <div style={{ flex: 1, minWidth: 200 }}>
          <label className="cms-label">Cerca per titolo</label>
          <input
            type="text"
            className="cms-input"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Titolo..."
          />
        </div>
        <div>
          <label htmlFor="content-filter-type" className="cms-label">
            Tipo
          </label>
          <select
            id="content-filter-type"
            className="cms-select"
            value={typeFilter}
            onChange={(e) => setTypeFilter(e.target.value as ContentType | "")}
          >
            <option value="">Tutti</option>
            {contentTypes.map((t) => (
              <option key={t} value={t}>
                {contentTypeLabel(t)}
              </option>
            ))}
          </select>
        </div>
        <div>
          <label htmlFor="content-filter-status" className="cms-label">
            Stato
          </label>
          <select
            id="content-filter-status"
            className="cms-select"
            value={statusFilter}
            onChange={(e) => setStatusFilter(e.target.value as PublicationStatus | "")}
          >
            <option value="">Tutti</option>
            {publicationStatuses.map((s) => (
              <option key={s} value={s}>
                {publicationStatusLabel(s)}
              </option>
            ))}
          </select>
        </div>
      </div>
      <div className="cms-card">
        {loading ? (
          <p className="cms-empty">Caricamento...</p>
        ) : filtered.length === 0 ? (
          <p className="cms-empty">Nessun contenuto trovato.</p>
        ) : (
          <table className="cms-table">
            <thead>
              <tr>
                <th>Titolo</th>
                <th>Tipo</th>
                <th>Stato</th>
                <th>Pubblicato il</th>
                <th style={{ width: 120 }}>Azioni</th>
              </tr>
            </thead>
            <tbody>
              {filtered.map((content) => (
                <tr key={content.id}>
                  <td>{content.title}</td>
                  <td>{contentTypeLabel(content.content_type)}</td>
                  <td>
                    <span className={`cms-badge cms-badge-${content.publication_status}`}>
                      {publicationStatusLabel(content.publication_status)}
                    </span>
                  </td>
                  <td>
                    {content.published_at
                      ? new Date(content.published_at).toLocaleDateString("it-IT")
                      : "—"}
                  </td>
                  <td>
                    <div className="cms-actions">
                      <button
                        type="button"
                        className="cms-button cms-button-secondary"
                        onClick={() => navigate(`/content/${content.id}`)}
                      >
                        Modifica
                      </button>
                      <button
                        type="button"
                        className="cms-button cms-button-danger"
                        onClick={() => setDeleteId(content.id)}
                      >
                        Elimina
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
      {deleteId && (
        <div className="cms-modal-overlay">
          <div className="cms-modal">
            <h2 className="cms-modal-title">Conferma eliminazione</h2>
            <p>
              Sei sicuro di voler eliminare questo contenuto? L'azione non è reversibile.
            </p>
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
