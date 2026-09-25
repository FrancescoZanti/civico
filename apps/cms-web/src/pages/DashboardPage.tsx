import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import * as api from "../api/api";
import "../styles.css";

export function DashboardPage() {
  const [counts, setCounts] = useState<{
    contents: number;
    published: number;
    drafts: number;
    homeItems: number;
    media: number;
  } | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    async function load() {
      try {
        const [contents, homeItems, media] = await Promise.all([
          api.listContent(),
          api.listHomeItems(),
          api.listMedia(),
        ]);
        if (cancelled) return;
        const published = contents.filter(
          (c) => c.publication_status === "published",
        ).length;
        const drafts = contents.filter((c) => c.publication_status === "draft").length;
        setCounts({
          contents: contents.length,
          published,
          drafts,
          homeItems: homeItems.length,
          media: media.length,
        });
      } catch {
        if (!cancelled) setError("Impossibile caricare i dati della dashboard.");
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
      <h1 className="cms-page-title">Dashboard</h1>
      {loading && <p className="cms-empty">Caricamento...</p>}
      {error && <div className="cms-form-error">{error}</div>}
      {counts && (
        <>
          <div className="cms-card">
            <div className="cms-grid cms-grid-4">
              <div className="cms-stat">
                <div className="cms-stat-value">{counts.published}</div>
                <div className="cms-stat-label">Contenuti pubblicati</div>
              </div>
              <div className="cms-stat">
                <div className="cms-stat-value">{counts.drafts}</div>
                <div className="cms-stat-label">Bozze</div>
              </div>
              <div className="cms-stat">
                <div className="cms-stat-value">{counts.homeItems}</div>
                <div className="cms-stat-label">Pulsanti HOME</div>
              </div>
              <div className="cms-stat">
                <div className="cms-stat-value">{counts.media}</div>
                <div className="cms-stat-label">File media</div>
              </div>
            </div>
          </div>
          <div className="cms-card">
            <h2 className="cms-card-title">Azioni rapide</h2>
            <div className="cms-actions">
              <Link to="/content/new" className="cms-button cms-button-primary">
                Nuovo contenuto
              </Link>
              <Link to="/home" className="cms-button cms-button-primary">
                Nuovo pulsante
              </Link>
              <Link to="/media" className="cms-button cms-button-primary">
                Carica media
              </Link>
              <Link to="/preview" className="cms-button cms-button-secondary">
                Apri anteprima
              </Link>
            </div>
          </div>
        </>
      )}
    </div>
  );
}
