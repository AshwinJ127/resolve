import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

function App() {
  const [status, setStatus] = useState("Loading...");
  const [log, setLog] = useState("");

  async function fetchStatus() {
    try {
      const result = await invoke("get_git_status");
      setStatus(result as string);
    } catch (e) {
      setStatus("Error: " + e);
    }
  }

  async function handleSync() {
    setLog("Syncing...");
    try {
      const result = await invoke("smart_sync");
      setLog(result as string);
      fetchStatus();
    } catch (e) {
      setLog("Error: " + e);
    }
  }

  useEffect(() => {
    fetchStatus();
  }, []);

  return (
    <div className="min-h-screen bg-gray-950 text-white flex flex-col items-center justify-center p-8">
      {/* Main Card */}
      <div className="w-full max-w-md bg-gray-900 border border-gray-800 rounded-xl shadow-2xl p-6">
        
        {/* Header */}
        <div className="flex items-center justify-between mb-6">
          <h1 className="text-xl font-bold bg-gradient-to-r from-blue-400 to-purple-500 bg-clip-text text-transparent">
            rfx Dashboard
          </h1>
          <span className="px-2 py-1 text-xs font-mono bg-gray-800 rounded text-gray-400">v0.1.0</span>
        </div>

        {/* Status Display */}
        <div className="bg-black/50 rounded-lg p-4 font-mono text-sm text-green-400 mb-6 border border-gray-800">
          <pre>{status}</pre>
        </div>

        {/* Actions Grid */}
        <div className="grid grid-cols-2 gap-3">
          <button 
            onClick={handleSync}
            className="flex items-center justify-center gap-2 bg-blue-600 hover:bg-blue-500 text-white py-2 px-4 rounded-lg font-medium transition-all active:scale-95"
          >
            <span>🔄</span> Sync
          </button>
          
          <button 
            onClick={fetchStatus}
            className="bg-gray-800 hover:bg-gray-700 text-gray-300 py-2 px-4 rounded-lg font-medium transition-all active:scale-95"
          >
            Refresh
          </button>
        </div>

        {/* Logs Area */}
        {log && (
          <div className="mt-4 p-3 bg-gray-800/50 rounded border border-gray-700 text-xs text-gray-400 font-mono break-words">
            {log}
          </div>
        )}
      </div>
    </div>
  );
}

export default App;