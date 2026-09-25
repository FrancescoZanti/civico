import { useEffect, useState } from "react";
import * as api from "../api/api";
import type { Media } from "../types";
import { API_BASE_URL } from "../types";
import "../styles.css";

interface MediaPickerProps {
  selectedId: string | null;
  onSelect: (id: string | null) => void;
}

export function MediaPicker({ selectedId, onSelect }: MediaPickerProps) {
  const [media, setMedia] = useState<Media[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let cancelled = false;
    async function load() {
      try {
        const data = await api.listMedia();
        if (!cancelled) setMedia(data);
      } finally {
        if (!cancelled) setLoading(false);
      }
    }
    load();
    return () => {
      cancelled = true;
    };
  }, []);

  if (loading) return <p>Caricamento media...</p>;
  if (media.length === 0) return <p className="cms-empty">Nessun media caricato.</p>;

  return (
    <div
      style={{
        display: "grid",
        gridTemplateColumns: "repeat(auto-fill, minmax(100px, 1fr))",
        gap: "0.75rem",
      }}
    >
      {media.map((m) => {
        const isSelected = m.id === selectedId;
        return (
          <button
            key={m.id}
            type="button"
            onClick={() => onSelect(isSelected ? null : m.id)}
            style={{
              border: isSelected ? "2px solid #0d6efd" : "1px solid #ced4da",
              borderRadius: "0.375rem",
              padding: "0.25rem",
              background: "#fff",
              cursor: "pointer",
              textAlign: "center",
            }}
          >
            <img
              src={`${API_BASE_URL}${m.url}`}
              alt={m.filename}
              style={{
                width: "100%",
                height: 80,
                objectFit: "cover",
                borderRadius: "0.25rem",
              }}
            />
            <div
              style={{
                fontSize: "0.75rem",
                color: "#495057",
                marginTop: "0.25rem",
                overflow: "hidden",
                textOverflow: "ellipsis",
                whiteSpace: "nowrap",
              }}
            >
              {m.filename}
            </div>
          </button>
        );
      })}
    </div>
  );
}
