export interface User { id: string; name: string; email: string }
export interface AuthResponse { token: string; user: User }
export interface Todo { id: string; title: string; completed: boolean; created_at: string; updated_at: string }
