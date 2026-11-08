import { create } from 'zustand';
import { api, errorMessage } from '../lib/api';
import { useAuth } from './auth';
import type { Todo } from '../types';
interface TodoState {
  todos: Todo[]; loading: boolean; error: string | null;
  fetch: () => Promise<void>; add: (title: string) => Promise<void>;
  update: (id: string, input: Partial<Pick<Todo, 'title' | 'completed'>>) => Promise<void>;
  remove: (id: string) => Promise<void>;
}
export const useTodos = create<TodoState>((set) => ({
  todos: [], loading: false, error: null,
  fetch: async () => {
    const token = useAuth.getState().token;
    set({ loading: true, error: null });
    try {
      const { data } = await api.get<Todo[]>('/todos');
      if (token === useAuth.getState().token) set({ todos: data });
    } catch (error) { if (token === useAuth.getState().token) set({ error: errorMessage(error) }); }
    finally { if (token === useAuth.getState().token) set({ loading: false }); }
  },
  add: async (title) => {
    const token = useAuth.getState().token;
    const { data } = await api.post<Todo>('/todos', { title });
    if (token === useAuth.getState().token) set((state) => ({ todos: [data, ...state.todos] }));
  },
  update: async (id, input) => {
    const token = useAuth.getState().token;
    const { data } = await api.patch<Todo>(`/todos/${id}`, input);
    if (token === useAuth.getState().token) set((state) => ({ todos: state.todos.map((todo) => todo.id === id ? data : todo) }));
  },
  remove: async (id) => {
    const token = useAuth.getState().token;
    await api.delete(`/todos/${id}`);
    if (token === useAuth.getState().token) set((state) => ({ todos: state.todos.filter((todo) => todo.id !== id) }));
  },
}));
useAuth.subscribe((state, previous) => {
  if (state.token !== previous.token) useTodos.setState({ todos: [], error: null, loading: false });
});
