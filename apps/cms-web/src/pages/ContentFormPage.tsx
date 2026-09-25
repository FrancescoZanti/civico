import { useEffect, useMemo, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import * as api from "../api/api";
import type { Content, ContentCreate, ContentType, PublicationStatus } from "../types";
import {
  ApiError,
  contentTypeLabel,
  contentTypes,
  publicationStatusLabel,
  publicationStatuses,
} from "../types";
import { MediaPicker } from "../components/MediaPicker";
import "../styles.css";

interface FormState {
  content_type: ContentType;
  title: string;
  slug: string;
  excerpt: string;
  body: string;
  publication_status: PublicationStatus;
  publish_from: string;
  publish_until: string;
  image_id: string;
  data: Record<string, unknown>;
}

function initialForm(): FormState {
  return {
    content_type: "news",
    title: "",
    slug: "",
    excerpt: "",
    body: "",
    publication_status: "draft",
    publish_from: "",
    publish_until: "",
    image_id: "",
    data: {},
  };
}

function contentToForm(content: Content): FormState {
  const data = (
    typeof content.body === "object" && content.body !== null ? content.body : {}
  ) as Record<string, unknown>;
  return {
    content_type: content.content_type,
    title: content.title,
    slug: content.slug,
    excerpt: content.excerpt ?? "",
    body: content.body ?? "",
    publication_status: content.publication_status,
    publish_from: content.publish_from ? toDatetimeLocal(content.publish_from) : "",
    publish_until: content.publish_until ? toDatetimeLocal(content.publish_until) : "",
    image_id: content.image_id ?? "",
    data,
  };
}

function toDatetimeLocal(iso: string): string {
  const d = new Date(iso);
  d.setMinutes(d.getMinutes() - d.getTimezoneOffset());
  return d.toISOString().slice(0, 16);
}

function fromDatetimeLocal(value: string): string | null {
  if (!value) return null;
  return new Date(value).toISOString();
}

function slugify(text: string): string {
  return text
    .toLowerCase()
    .trim()
    .replace(/[^a-z0-9\s-]/g, "")
    .replace(/\s+/g, "-")
    .replace(/-+/g, "-");
}

export function ContentFormPage() {
  const { id } = useParams<{ id: string }>();
  const currentId = id ?? "";
  const navigate = useNavigate();
  const isEdit = Boolean(id);
  const [form, setForm] = useState<FormState>(initialForm);
  const [loading, setLoading] = useState(isEdit);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [fieldErrors, setFieldErrors] = useState<Record<string, string>>({});
  const [success, setSuccess] = useState(false);

  useEffect(() => {
    if (!currentId) return;
    let cancelled = false;
    async function load() {
      try {
        const content = await api.getContent(currentId);
        if (!cancelled) setForm(contentToForm(content));
      } catch {
        if (!cancelled) setError("Impossibile caricare il contenuto.");
      } finally {
        if (!cancelled) setLoading(false);
      }
    }
    load();
    return () => {
      cancelled = true;
    };
  }, [currentId]);

  const validate = (): boolean => {
    const errors: Record<string, string> = {};
    if (!form.title.trim()) errors.title = "Il titolo è obbligatorio.";
    if (!form.slug.trim()) errors.slug = "Lo slug è obbligatorio.";
    else if (!/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(form.slug))
      errors.slug =
        "Lo slug può contenere solo lettere minuscole, numeri e trattini singoli.";
    if (
      form.publish_from &&
      form.publish_until &&
      new Date(form.publish_from) >= new Date(form.publish_until)
    ) {
      errors.publish_until =
        "La data di fine deve essere successiva alla data di inizio.";
    }
    setFieldErrors(errors);
    return Object.keys(errors).length === 0;
  };

  const buildPayload = (): ContentCreate => {
    const payload: ContentCreate = {
      content_type: form.content_type,
      title: form.title.trim(),
      slug: form.slug.trim(),
      excerpt: form.excerpt.trim() || null,
      body: form.body.trim() || null,
      publication_status: form.publication_status,
      publish_from: fromDatetimeLocal(form.publish_from),
      publish_until: fromDatetimeLocal(form.publish_until),
      image_id: form.image_id || null,
      data: form.data,
    };
    return payload;
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setSuccess(false);
    if (!validate()) return;
    setSaving(true);
    try {
      const payload = buildPayload();
      if (id) {
        await api.updateContent(id, payload);
      } else {
        await api.createContent(payload);
      }
      setSuccess(true);
      if (!id) setForm(initialForm());
    } catch (err) {
      if (err instanceof ApiError) {
        setError(err.message);
      } else {
        setError("Impossibile salvare il contenuto.");
      }
    } finally {
      setSaving(false);
    }
  };

  const updateField = <K extends keyof FormState>(key: K, value: FormState[K]) => {
    setForm((prev) => ({ ...prev, [key]: value }));
    setFieldErrors((prev) => ({ ...prev, [key]: "" }));
  };

  const updateDataField = (key: string, value: unknown) => {
    setForm((prev) => ({ ...prev, data: { ...prev.data, [key]: value } }));
  };

  const typeSpecificFields = useMemo(() => {
    switch (form.content_type) {
      case "event":
        return (
          <>
            <div className="cms-form-group">
              <label className="cms-label">Data e ora inizio</label>
              <input
                type="datetime-local"
                className="cms-input"
                value={((form.data.start_at as string | undefined) ?? "").slice(0, 16)}
                onChange={(e) =>
                  updateDataField(
                    "start_at",
                    e.target.value ? new Date(e.target.value).toISOString() : undefined,
                  )
                }
              />
            </div>
            <div className="cms-form-group">
              <label className="cms-label">Data e ora fine (opzionale)</label>
              <input
                type="datetime-local"
                className="cms-input"
                value={((form.data.end_at as string | undefined) ?? "").slice(0, 16)}
                onChange={(e) =>
                  updateDataField(
                    "end_at",
                    e.target.value ? new Date(e.target.value).toISOString() : undefined,
                  )
                }
              />
            </div>
            <div className="cms-form-group">
              <label className="cms-label">Luogo (opzionale)</label>
              <input
                type="text"
                className="cms-input"
                value={((form.data.location as string) ?? "").toString()}
                onChange={(e) => updateDataField("location", e.target.value || undefined)}
              />
            </div>
          </>
        );
      case "place":
        return (
          <div className="cms-form-group">
            <label className="cms-label">Indirizzo (opzionale)</label>
            <input
              type="text"
              className="cms-input"
              value={((form.data.address as string) ?? "").toString()}
              onChange={(e) => updateDataField("address", e.target.value || undefined)}
            />
          </div>
        );
      case "contact":
        return (
          <>
            <div className="cms-form-group">
              <label className="cms-label">Telefono (opzionale)</label>
              <input
                type="text"
                className="cms-input"
                value={((form.data.phone as string) ?? "").toString()}
                onChange={(e) => updateDataField("phone", e.target.value || undefined)}
              />
            </div>
            <div className="cms-form-group">
              <label className="cms-label">Email (opzionale)</label>
              <input
                type="email"
                className="cms-input"
                value={((form.data.email as string) ?? "").toString()}
                onChange={(e) => updateDataField("email", e.target.value || undefined)}
              />
            </div>
            <div className="cms-form-group">
              <label className="cms-label">Sito web (opzionale)</label>
              <input
                type="text"
                className="cms-input"
                value={((form.data.website as string) ?? "").toString()}
                onChange={(e) => updateDataField("website", e.target.value || undefined)}
              />
            </div>
          </>
        );
      default:
        return null;
    }
  }, [form.content_type, form.data]);

  if (loading) return <p className="cms-empty">Caricamento...</p>;

  return (
    <div>
      <h1 className="cms-page-title">
        {isEdit ? "Modifica contenuto" : "Nuovo contenuto"}
      </h1>
      <div className="cms-card">
        {success && (
          <div className="cms-form-success">Contenuto salvato con successo.</div>
        )}
        {error && <div className="cms-form-error">{error}</div>}
        <form onSubmit={handleSubmit}>
          <div className="cms-form-group">
            <label className="cms-label">Tipo</label>
            <select
              className="cms-select"
              value={form.content_type}
              onChange={(e) => updateField("content_type", e.target.value as ContentType)}
            >
              {contentTypes.map((t) => (
                <option key={t} value={t}>
                  {contentTypeLabel(t)}
                </option>
              ))}
            </select>
          </div>
          <div className="cms-form-group">
            <label htmlFor="content-title" className="cms-label">
              Titolo
            </label>
            <input
              id="content-title"
              type="text"
              className="cms-input"
              value={form.title}
              onChange={(e) => {
                updateField("title", e.target.value);
                if (!isEdit) updateField("slug", slugify(e.target.value));
              }}
              required
            />
            {fieldErrors.title && (
              <div className="cms-field-error">{fieldErrors.title}</div>
            )}
          </div>
          <div className="cms-form-group">
            <label htmlFor="content-slug" className="cms-label">
              Slug
            </label>
            <input
              id="content-slug"
              type="text"
              className="cms-input"
              value={form.slug}
              onChange={(e) => updateField("slug", e.target.value)}
              required
            />
            {fieldErrors.slug && (
              <div className="cms-field-error">{fieldErrors.slug}</div>
            )}
          </div>
          <div className="cms-form-group">
            <label htmlFor="content-excerpt" className="cms-label">
              Sottotitolo / Estratto
            </label>
            <input
              id="content-excerpt"
              type="text"
              className="cms-input"
              value={form.excerpt}
              onChange={(e) => updateField("excerpt", e.target.value)}
            />
          </div>
          <div className="cms-form-group">
            <label htmlFor="content-body" className="cms-label">
              Corpo
            </label>
            <textarea
              id="content-body"
              className="cms-textarea"
              value={form.body}
              onChange={(e) => updateField("body", e.target.value)}
            />
          </div>
          {typeSpecificFields}
          <div className="cms-form-group">
            <label className="cms-label">Immagine in evidenza</label>
            <MediaPicker
              selectedId={form.image_id || null}
              onSelect={(id) => updateField("image_id", id ?? "")}
            />
          </div>
          <div className="cms-form-group">
            <label className="cms-label">Stato pubblicazione</label>
            <select
              className="cms-select"
              value={form.publication_status}
              onChange={(e) =>
                updateField("publication_status", e.target.value as PublicationStatus)
              }
            >
              {publicationStatuses.map((s) => (
                <option key={s} value={s}>
                  {publicationStatusLabel(s)}
                </option>
              ))}
            </select>
          </div>
          <div className="cms-form-group">
            <label className="cms-label">Pubblica da (opzionale)</label>
            <input
              type="datetime-local"
              className="cms-input"
              value={form.publish_from}
              onChange={(e) => updateField("publish_from", e.target.value)}
            />
          </div>
          <div className="cms-form-group">
            <label className="cms-label">Pubblica fino a (opzionale)</label>
            <input
              type="datetime-local"
              className="cms-input"
              value={form.publish_until}
              onChange={(e) => updateField("publish_until", e.target.value)}
            />
            {fieldErrors.publish_until && (
              <div className="cms-field-error">{fieldErrors.publish_until}</div>
            )}
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
              onClick={() => navigate("/content")}
              disabled={saving}
            >
              Annulla
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
