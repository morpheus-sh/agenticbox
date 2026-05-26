import { create } from "zustand";

interface Session {
  id: string;
  name: string;
}

interface SessionStore {
  sessions: Session[];
  addSession: (s: Session) => void;
  removeSession: (id: string) => void;
}

export const useSessionStore = create<SessionStore>((set) => ({
  sessions: [],
  addSession: (s) => set((state) => ({ sessions: [...state.sessions, s] })),
  removeSession: (id) => set((state) => ({
    sessions: state.sessions.filter((s) => s.id !== id),
  })),
}));
