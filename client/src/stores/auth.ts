import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';
import { api } from '../lib/api';
import type { User, AuthResponse } from '../types';
interface AuthState {
  token: string | null; user: User | null;
  authenticate: (mode: 'login' | 'signup', input: { email: string; password: string; name?: string }) => Promise<void>;
  logout: () => void;
}
export const useAuth = create<AuthState>()(persist((set) => ({
  token: null, user: null,
  authenticate: async (mode, input) => {
    const { data } = await api.post<AuthResponse>(`/auth/${mode}`, input);
    set({ token: data.token, user: data.user });
  },
  logout: () => set({ token: null, user: null }),
}), { name: 'daylist-auth', storage: createJSONStorage(() => sessionStorage), partialize: ({ token, user }) => ({ token, user }) }));
api.interceptors.request.use((config) => {
  const token = useAuth.getState().token;
  if (token) config.headers.Authorization = `Bearer ${token}`;
  return config;
});
api.interceptors.response.use((response) => response, (error) => {
  if (error.response?.status === 401 && !error.config?.url?.includes('/auth/login')) useAuth.getState().logout();
  return Promise.reject(error);
});
