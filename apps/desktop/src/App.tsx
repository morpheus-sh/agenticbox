import { useState } from "react";
import { useSessionStore } from "./store";

function App() {
  const sessions = useSessionStore((s) => s.sessions);
  const addSession = useSessionStore((s) => s.addSession);
  const [name, setName] = useState("");

  return (
    <div className="min-h-screen p-6">
      <h1 className="text-3xl font-bold mb-4">Local AI Agent Runtime</h1>
      <div className="flex gap-2 mb-6">
        <input
          className="border border-gray-700 bg-gray-800 rounded px-3 py-2"
          placeholder="Session name"
          value={name}
          onChange={(e) => setName(e.target.value)}
        />
        <button
          className="bg-blue-600 hover:bg-blue-500 px-4 py-2 rounded"
          onClick={() => {
            addSession({ id: crypto.randomUUID(), name });
            setName("");
          }}
        >
          New Session
        </button>
      </div>
      <div className="grid gap-4">
        {sessions.map((s) => (
          <div key={s.id} className="bg-gray-800 p-4 rounded border border-gray-700">
            <div className="font-semibold">{s.name}</div>
            <div className="text-sm text-gray-400">ID: {s.id}</div>
          </div>
        ))}
      </div>
    </div>
  );
}

export default App;
