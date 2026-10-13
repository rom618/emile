import { useEffect, useState } from "react";
import { addItems, listItems, parseText, type Item, type NewItem, type Place } from "./api";

const PLACES: Place[] = ["fridge", "freezer", "cupboard"];

function daysLeft(date: string | null): number | null {
  if (!date) return null;
  const ms = new Date(date + "T00:00:00").getTime() - new Date(new Date().toDateString()).getTime();
  return Math.round(ms / 86400000);
}

function ExpiryBadge({ date, guessed }: { date: string | null; guessed: boolean }) {
  const d = daysLeft(date);
  if (d === null) return null;
  const tone = d < 0 ? "bg-red-600" : d <= 2 ? "bg-amber-600" : "bg-stone-700";
  const label = d < 0 ? "expired" : d === 0 ? "today" : `${d}d`;
  return <span className={`rounded px-1.5 py-0.5 text-xs ${tone}`}>{guessed ? "~" : ""}{label}</span>;
}

function amount(i: NewItem): string {
  const base = `${i.qty} ${i.unit}`;
  if (!i.cook_unit || !i.cook_per_unit) return base;
  return `${base} (about ${Math.round(i.qty * i.cook_per_unit)} ${i.cook_unit})`;
}

function Review({ draft, setDraft, onSave, onCancel, busy }: {
  draft: NewItem[];
  setDraft: (d: NewItem[]) => void;
  onSave: () => void;
  onCancel: () => void;
  busy: boolean;
}) {
  const edit = (idx: number, patch: Partial<NewItem>) => setDraft(draft.map((d, i) => (i === idx ? { ...d, ...patch } : d)));
  return (
    <section className="space-y-2">
      <h2 className="text-sm uppercase tracking-wide text-stone-400">Check before saving</h2>
      {draft.map((d, idx) => (
        <div key={idx} className="grid grid-cols-6 gap-1 rounded-lg bg-stone-800 p-2 text-sm">
          <input className="col-span-3 rounded bg-stone-700 px-2 py-1" value={d.name} onChange={(e) => edit(idx, { name: e.target.value })} />
          <input className="col-span-1 rounded bg-stone-700 px-2 py-1" inputMode="decimal" value={d.qty} onChange={(e) => edit(idx, { qty: Number(e.target.value) || 0 })} />
          <input className="col-span-1 rounded bg-stone-700 px-2 py-1" value={d.unit} onChange={(e) => edit(idx, { unit: e.target.value })} />
          <button className="col-span-1 rounded bg-stone-700" onClick={() => setDraft(draft.filter((_, i) => i !== idx))}>x</button>
          <select className="col-span-3 rounded bg-stone-700 px-2 py-1" value={d.place} onChange={(e) => edit(idx, { place: e.target.value as Place })}>
            {PLACES.map((p) => <option key={p}>{p}</option>)}
          </select>
          <input className="col-span-3 rounded bg-stone-700 px-2 py-1" type="date" value={d.expires_on ?? ""} onChange={(e) => edit(idx, { expires_on: e.target.value || null, expiry_guessed: false })} />
        </div>
      ))}
      <div className="flex gap-2">
        <button className="flex-1 rounded-lg bg-amber-500 py-2 font-medium text-stone-900 disabled:opacity-50" disabled={busy || draft.length === 0} onClick={onSave}>Save {draft.length}</button>
        <button className="rounded-lg bg-stone-700 px-4 py-2" onClick={onCancel}>Cancel</button>
      </div>
    </section>
  );
}

export default function App() {
  const [items, setItems] = useState<Item[]>([]);
  const [text, setText] = useState("");
  const [draft, setDraft] = useState<NewItem[] | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = () => listItems().then(setItems).catch((e) => setError(String(e.message ?? e)));
  useEffect(() => { refresh(); }, []);

  const run = async (fn: () => Promise<void>) => {
    setBusy(true);
    setError(null);
    try { await fn(); } catch (e) { setError(e instanceof Error ? e.message : String(e)); } finally { setBusy(false); }
  };

  const parse = () => run(async () => setDraft((await parseText(text)).items));
  const save = () => run(async () => {
    await addItems(draft ?? [], "text");
    setDraft(null);
    setText("");
    await refresh();
  });

  return (
    <main className="mx-auto max-w-md space-y-5 p-4">
      <h1 className="text-2xl font-semibold">Pantry</h1>
      {error && <p className="rounded bg-red-900/60 p-2 text-sm">{error}</p>}
      {draft ? (
        <Review draft={draft} setDraft={setDraft} onSave={save} onCancel={() => setDraft(null)} busy={busy} />
      ) : (
        <section className="space-y-2">
          <textarea className="h-24 w-full rounded-lg bg-stone-800 p-3" placeholder="What did you buy? e.g. 500 g potatoes, 6 eggs, a pack of bacon" value={text} onChange={(e) => setText(e.target.value)} />
          <button className="w-full rounded-lg bg-amber-500 py-2 font-medium text-stone-900 disabled:opacity-50" disabled={busy || !text.trim()} onClick={parse}>{busy ? "Reading..." : "Add to pantry"}</button>
        </section>
      )}
      {PLACES.map((place) => {
        const here = items.filter((i) => i.place === place);
        if (here.length === 0) return null;
        return (
          <section key={place} className="space-y-1">
            <h2 className="text-sm uppercase tracking-wide text-stone-400">{place}</h2>
            {here.map((i) => (
              <div key={i.id} className="flex items-center justify-between rounded-lg bg-stone-800 px-3 py-2">
                <div>
                  <div>{i.name}</div>
                  <div className="text-xs text-stone-400">{amount(i)}</div>
                </div>
                <ExpiryBadge date={i.expires_on} guessed={i.expiry_guessed} />
              </div>
            ))}
          </section>
        );
      })}
      {items.length === 0 && !draft && <p className="text-center text-stone-500">The pantry is empty.</p>}
    </main>
  );
}
