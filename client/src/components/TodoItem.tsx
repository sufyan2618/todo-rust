import { useState, type FormEvent } from 'react';
import { Check, Pencil, Trash2, X } from 'lucide-react';
import { useTodos } from '../stores/todos';
import { errorMessage } from '../lib/api';
import type { Todo } from '../types';
export function TodoItem({ todo }: { todo: Todo }) {
  const { update, remove } = useTodos();
  const [editing, setEditing] = useState(false);
  const [title, setTitle] = useState(todo.title);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  async function act(action: () => Promise<void>) {
    setBusy(true); setError('');
    try { await action(); setEditing(false); } catch (error) { setError(errorMessage(error)); }
    finally { setBusy(false); }
  }
  function save(event: FormEvent) { event.preventDefault(); if (title.trim()) void act(() => update(todo.id, { title: title.trim() })); }
  return <li className={`todo-item ${todo.completed ? 'is-completed' : ''}`}>
    <div className="todo-row">
      <button className="checkbox" aria-label={todo.completed ? `Mark ${todo.title} active` : `Complete ${todo.title}`} aria-pressed={todo.completed} disabled={busy} onClick={() => void act(() => update(todo.id, { completed: !todo.completed }))}>{todo.completed && <Check size={15}/>}</button>
      {editing ? <form className="edit-form" onSubmit={save}><input autoFocus aria-label="Edit task title" value={title} maxLength={200} onChange={(event) => setTitle(event.target.value)} required/><button disabled={busy || !title.trim()} className="text-button">Save</button><button type="button" className="icon-button" aria-label="Cancel edit" onClick={() => setEditing(false)}><X size={16}/></button></form> : <><span className="todo-title">{todo.title}</span><span className="todo-actions"><button className="icon-button" aria-label={`Edit ${todo.title}`} disabled={busy} onClick={() => { setTitle(todo.title); setEditing(true); }}><Pencil size={15}/></button><button className="icon-button delete" aria-label={`Delete ${todo.title}`} disabled={busy} onClick={() => void act(() => remove(todo.id))}><Trash2 size={15}/></button></span></>}
    </div>{error && <p className="error" role="alert">{error}</p>}
  </li>;
}
