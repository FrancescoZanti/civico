import { useEffect, useState } from "react";
import * as api from "../api/api";
import type { Content, HomeItem, HomeItemCreate, HomeItemUpdate } from "../types";
import {
  ApiError,
  contentTypeLabel,
  contentTypes,
  destinationLabel,
  destinationTypeLabel,
} from "../types";
import { MediaPicker } from "../components/MediaPicker";
import "../styles.css";

type EditingItem = HomeItem | null;

export function HomeEditorPage() {
  const [items, setItems] = useState<HomeItem[]>([]);
  const [contents, setContents] = useState<Content[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [editing, setEditing] = useState<EditingItem>(null);
  const [isCreating, setIsCreating] = useState(false);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    let cancelled = false;
    async function load() {
      try {
        const [itemsData, contentsData] = await Promise.all([
          api.listHomeItems(),
          api.listContent(),
        ]);
        if (!cancelled) {
          setItems(itemsData.sort((a, b) => a.position - b.position));
          setContents(contentsData);
        }
      } catch {
        if (!cancelled) setError("Impossibile caricare i dati.");
      } finally {
        if (!cancelled) setLoading(false);
      }
    }
    load();
    return () => {
      cancelled = true;
    };
  }, []);

  const saveItem = async (payload: HomeItemCreate | HomeItemUpdate, id?: string) => {
    setSaving(true);
    try {
      if (id) {
        await api.updateHomeItem(id, payload);
      } else {
        await api.createHomeItem(payload as HomeItemCreate);
      }
      const updated = await api.listHomeItems();
      setItems(updated.sort((a, b) => a.position - b.position));
      setEditing(null);
      setIsCreating(false);
    } catch (err) {
      if (err instanceof ApiError) setError(err.message);
      else setError("Impossibile salvare il pulsante.");
    } finally {
      setSaving(false);
    }
  };

  const moveItem = async (index: number, direction: -1 | 1) => {
    const target = index + direction;
    if (target < 0 || target >= items.length) return;
    const newItems = [...items];
    const temp = newItems[index]!;
    newItems[index] = newItems[target]!;
    newItems[target] = temp;
    const updates = newItems.map((item, i) => ({ ...item, position: i }));
    setItems(updates);
    try {
      await Promise.all(
        updates.map((item) => api.updateHomeItem(item.id, { position: item.position })),
      );
    } catch {
      setError("Impossibile aggiornare l'ordine.");
    }
  };

  const toggleEnabled = async (item: HomeItem) => {
    try {
      await api.updateHomeItem(item.id, { enabled: !item.enabled });
      setItems((prev) =>
        prev.map((i) => (i.id === item.id ? { ...i, enabled: !i.enabled } : i)),
      );
    } catch {
      setError("Impossibile aggiornare lo stato.");
    }
  };

  const deleteItem = async (id: string) => {
    try {
      await api.deleteHomeItem(id);
      setItems((prev) => prev.filter((i) => i.id !== id));
    } catch {
      setError("Impossibile eliminare il pulsante.");
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
          Home
        </h1>
        <button
          type="button"
          className="cms-button cms-button-primary"
          onClick={() => setIsCreating(true)}
        >
          Nuovo pulsante
        </button>
      </div>
      {error && <div className="cms-form-error">{error}</div>}
      {(isCreating || editing) && (
        <HomeItemForm
          item={editing}
          contents={contents}
          saving={saving}
          onSave={saveItem}
          onCancel={() => {
            setEditing(null);
            setIsCreating(false);
          }}
        />
      )}
      <div className="cms-card">
        {loading ? (
          <p className="cms-empty">Caricamento...</p>
        ) : items.length === 0 ? (
          <p className="cms-empty">Nessun pulsante HOME. Crea il primo pulsante.</p>
        ) : (
          <ul style={{ listStyle: "none", padding: 0, margin: 0 }}>
            {items.map((item, index) => (
              <li
                key={item.id}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: "1rem",
                  padding: "1rem 0",
                  borderBottom: "1px solid #dee2e6",
                  opacity: item.enabled ? 1 : 0.6,
                }}
              >
                <div style={{ fontSize: "1.5rem" }}>{item.icon || "🔘"}</div>
                <div style={{ flex: 1 }}>
                  <div style={{ fontWeight: 600 }}>
                    {item.title}
                    {!item.enabled && (
                      <span
                        className="cms-badge cms-badge-draft"
                        style={{ marginLeft: "0.5rem" }}
                      >
                        Disabilitato
                      </span>
                    )}
                  </div>
                  <div style={{ fontSize: "0.875rem", color: "#6c757d" }}>
                    {item.subtitle}
                  </div>
                  <div style={{ fontSize: "0.875rem", color: "#0d6efd" }}>
                    {destinationLabel(item, contents)}
                  </div>
                </div>
                <div className="cms-actions">
                  <button
                    type="button"
                    className="cms-button cms-button-ghost"
                    onClick={() => moveItem(index, -1)}
                    disabled={index === 0}
                  >
                    Su
                  </button>
                  <button
                    type="button"
                    className="cms-button cms-button-ghost"
                    onClick={() => moveItem(index, 1)}
                    disabled={index === items.length - 1}
                  >
                    Giu
                  </button>
                  <button
                    type="button"
                    className="cms-button cms-button-secondary"
                    onClick={() => toggleEnabled(item)}
                  >
                    {item.enabled ? "Disabilita" : "Abilita"}
                  </button>
                  <button
                    type="button"
                    className="cms-button cms-button-secondary"
                    onClick={() => setEditing(item)}
                  >
                    Modifica
                  </button>
                  <button
                    type="button"
                    className="cms-button cms-button-danger"
                    onClick={() => deleteItem(item.id)}
                  >
                    Elimina
                  </button>
                </div>
              </li>
            ))}
          </ul>
        )}
      </div>
    </div>
  );
}

interface HomeItemFormProps {
  item: HomeItem | null;
  contents: Content[];
  saving: boolean;
  onSave: (payload: HomeItemCreate | HomeItemUpdate, id?: string) => void;
  onCancel: () => void;
}

function HomeItemForm({ item, contents, saving, onSave, onCancel }: HomeItemFormProps) {
  const [title, setTitle] = useState(item?.title ?? "");
  const [subtitle, setSubtitle] = useState(item?.subtitle ?? "");
  const [icon, setIcon] = useState(item?.icon ?? "");
  const [imageId, setImageId] = useState<string | null>(item?.image_id ?? null);
  const [destinationType, setDestinationType] = useState<"section" | "content">(
    item?.destination_type ?? "section",
  );
  const [sectionType, setSectionType] = useState<string>(
    item?.destination_section_type ?? "news",
  );
  const [contentId, setContentId] = useState<string>(item?.destination_content_id ?? "");
  const [enabled, setEnabled] = useState(item?.enabled ?? true);
  const [fieldErrors, setFieldErrors] = useState<Record<string, string>>({});

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const errors: Record<string, string> = {};
    if (!title.trim()) errors.title = "Il titolo è obbligatorio.";
    if (destinationType === "section" && !sectionType)
      errors.sectionType = "Seleziona una sezione.";
    if (destinationType === "content" && !contentId)
      errors.contentId = "Seleziona un contenuto.";
    setFieldErrors(errors);
    if (Object.keys(errors).length > 0) return;

    const payload: HomeItemCreate = {
      title: title.trim(),
      subtitle: subtitle.trim() || null,
      icon: icon.trim() || null,
      image_id: imageId,
      enabled,
      destination_type: destinationType,
      destination_section_type:
        destinationType === "section"
          ? (sectionType as import("../types").ContentType)
          : null,
      destination_content_id: destinationType === "content" ? contentId : null,
    };

    onSave(payload, item?.id);
  };

  return (
    <div className="cms-card" style={{ marginBottom: "1.5rem" }}>
      <h2 className="cms-card-title">{item ? "Modifica pulsante" : "Nuovo pulsante"}</h2>
      <form onSubmit={handleSubmit}>
        <div className="cms-form-group">
          <label htmlFor="home-item-title" className="cms-label">
            Titolo
          </label>
          <input
            id="home-item-title"
            type="text"
            className="cms-input"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            required
          />
          {fieldErrors.title && (
            <div className="cms-field-error">{fieldErrors.title}</div>
          )}
        </div>
        <div className="cms-form-group">
          <label htmlFor="home-item-subtitle" className="cms-label">
            Sottotitolo
          </label>
          <input
            id="home-item-subtitle"
            type="text"
            className="cms-input"
            value={subtitle}
            onChange={(e) => setSubtitle(e.target.value)}
          />
        </div>
        <div className="cms-form-group">
          <label htmlFor="home-item-icon" className="cms-label">
            Icona (emoji o testo)
          </label>
          <input
            id="home-item-icon"
            type="text"
            className="cms-input"
            value={icon}
            onChange={(e) => setIcon(e.target.value)}
            placeholder="es. 🏛️"
          />
        </div>
        <div className="cms-form-group">
          <label className="cms-label">Immagine</label>
          <MediaPicker selectedId={imageId} onSelect={setImageId} />
        </div>
        <div className="cms-form-group">
          <label htmlFor="home-item-destination-type" className="cms-label">
            Destinazione
          </label>
          <select
            id="home-item-destination-type"
            className="cms-select"
            value={destinationType}
            onChange={(e) => setDestinationType(e.target.value as "section" | "content")}
          >
            <option value="section">{destinationTypeLabel("section")}</option>
            <option value="content">{destinationTypeLabel("content")}</option>
          </select>
        </div>
        {destinationType === "section" && (
          <div className="cms-form-group">
            <label htmlFor="home-item-section" className="cms-label">
              Sezione
            </label>
            <select
              id="home-item-section"
              className="cms-select"
              value={sectionType}
              onChange={(e) => setSectionType(e.target.value)}
            >
              {contentTypes.map((t) => (
                <option key={t} value={t}>
                  {contentTypeLabel(t)}
                </option>
              ))}
            </select>
            {fieldErrors.sectionType && (
              <div className="cms-field-error">{fieldErrors.sectionType}</div>
            )}
          </div>
        )}
        {destinationType === "content" && (
          <div className="cms-form-group">
            <label htmlFor="home-item-content" className="cms-label">
              Contenuto
            </label>
            <select
              id="home-item-content"
              className="cms-select"
              value={contentId}
              onChange={(e) => setContentId(e.target.value)}
            >
              <option value="">Seleziona...</option>
              {contents.map((c) => (
                <option key={c.id} value={c.id}>
                  {c.title}
                </option>
              ))}
            </select>
            {fieldErrors.contentId && (
              <div className="cms-field-error">{fieldErrors.contentId}</div>
            )}
          </div>
        )}
        <div className="cms-form-group">
          <label
            className="cms-label"
            style={{ display: "flex", alignItems: "center", gap: "0.5rem" }}
          >
            <input
              type="checkbox"
              checked={enabled}
              onChange={(e) => setEnabled(e.target.checked)}
            />
            Abilitato
          </label>
        </div>
        <div className="cms-actions">
          <button
            type="submit"
            className="cms-button cms-button-primary"
            disabled={saving}
          >
            {saving ? "Salvataggio..." : "Salva"}
          </button>
          <button
            type="button"
            className="cms-button cms-button-secondary"
            onClick={onCancel}
            disabled={saving}
          >
            Annulla
          </button>
        </div>
      </form>
    </div>
  );
}
